cd ~/phase2_bot
export $(cat .env | xargs)
echo -e "1\n40\n0\n" | ./target/release/nonce-manager
