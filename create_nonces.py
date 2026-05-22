
import json, subprocess, time

# We need to create nonce accounts using solana CLI or the bot's built-in system
# Since solana CLI isn't installed, we'll use the bot binary's CLI mode

# Actually, the simplest approach: empty the nonce_accounts.json and let the bot
# create new ones. OR we use Python + solders to create them.

try:
    from solders.keypair import Keypair
    from solders.pubkey import Pubkey
    from solders.system_program import CreateNonceAccountParams, create_nonce_account
    from solders.transaction import Transaction
    from solders.message import Message
    import base58
    import urllib.request
    
    RPC = "https://rpc.shyft.to?api_key=Bs3GIL0Q3NdkzFNc"
    
    def rpc_call(method, params):
        data = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode()
        req = urllib.request.Request(RPC, data=data, headers={"Content-Type": "application/json"})
        resp = urllib.request.urlopen(req, timeout=30)
        return json.loads(resp.read())
    
    # Load signer
    pk_str = "Wsy8bV7rrZZtxngAcaqJSgcrXvdMA3xeCYSDRxr6sZMdwnC1qiuQVREftAfszd2rwYYQBgo2cfVs7hMnKd98HZW"
    signer = Keypair.from_bytes(base58.b58decode(pk_str))
    print(f"Signer: {signer.pubkey()}")
    
    # Check balance
    result = rpc_call("getBalance", [str(signer.pubkey())])
    balance = result["result"]["value"]
    print(f"Balance: {balance / 1e9:.6f} SOL")
    
    NONCE_RENT = 1447680  # Rent for nonce account
    COUNT = 20
    
    if balance < (NONCE_RENT + 5000) * COUNT:
        print(f"NOT ENOUGH SOL! Need ~{(NONCE_RENT + 5000) * COUNT / 1e9:.4f} SOL")
        exit(1)
    
    accounts = []
    for i in range(COUNT):
        nonce_kp = Keypair()
        nonce_pub = nonce_kp.pubkey()
        
        # Create nonce account instructions
        ixs = create_nonce_account(
            CreateNonceAccountParams(
                from_pubkey=signer.pubkey(),
                nonce_pubkey=nonce_pub,
                authority=signer.pubkey(),
                lamports=NONCE_RENT,
            )
        )
        
        # Get blockhash
        bh_result = rpc_call("getLatestBlockhash", [{"commitment": "finalized"}])
        blockhash = bh_result["result"]["value"]["blockhash"]
        
        from solders.hash import Hash
        msg = Message.new_with_blockhash(ixs, signer.pubkey(), Hash.from_string(blockhash))
        tx = Transaction.new_unsigned(msg)
        tx.sign([signer, nonce_kp], Hash.from_string(blockhash))
        
        # Send
        tx_bytes = bytes(tx)
        import base64 as b64
        tx_b64 = b64.b64encode(tx_bytes).decode()
        
        send_result = rpc_call("sendTransaction", [tx_b64, {"encoding": "base64", "skipPreflight": False, "preflightCommitment": "processed"}])
        
        if "result" in send_result:
            sig = send_result["result"]
            accounts.append(str(nonce_pub))
            print(f"  [{i+1}/{COUNT}] Created: {nonce_pub} (sig: {sig[:20]}...)")
            time.sleep(1)  # Rate limit
        else:
            err = send_result.get("error", {})
            print(f"  [{i+1}/{COUNT}] FAILED: {err}")
            time.sleep(2)
    
    # Save to file
    if accounts:
        with open("/root/phase2_bot/nonce_accounts.json", "w") as f:
            json.dump({"accounts": accounts}, f, indent=2)
        print(f"\nSaved {len(accounts)} nonce accounts to file")
    else:
        print("\nNo accounts created!")
        
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Installing solders...")
    import subprocess
    subprocess.run(["pip3", "install", "solders", "base58"], check=True)
    print("Installed! Run this script again.")
