use solana_sdk::{pubkey, pubkey::Pubkey};

pub const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

//token program address
pub const ASSOCIATED_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const TOKEN_2022_PROGRAM: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

//seeds
pub static USER_VOLUME_ACCUMULATOR_SEED: &[u8] = b"user_volume_accumulator";
pub static GLOBAL_ACCUMULATOR_SEED: &[u8] = b"global_volume_accumulator";
pub static PUMPFUN_CREATOR_VAULT_SEED: &[u8] = b"creator-vault";
pub static PUMPSWAP_CREATOR_VAULT_SEED: &[u8] = b"creator_vault";
pub static POOL_SEED: &[u8] = b"pool";
pub static PUMPFUN_BONDING_CURVE: &[u8] = b"bonding-curve";
pub static PUMPFUN_BONDING_CURVE_V2_SEED: &[u8] = b"bonding-curve-v2";
pub static PUMPFUN_POOL_AUTH: &[u8] = b"pool-authority";
pub static PUMPSWAP_POOL_V2_SEED: &[u8] = b"pool-v2";

//fee program
pub const BUDGET_COMPUTE_PROGRAM: Pubkey = pubkey!("ComputeBudget111111111111111111111111111111");
//system program
pub const SYSTEM_PROGRAM: Pubkey = pubkey!("11111111111111111111111111111111");

//pumpfun accounts
pub const PUMPFUN_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const PUMPFUN_FEE_RECIPIENT: Pubkey = pubkey!("62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
pub const PUMPFUN_FEE_CONFIG: Pubkey = pubkey!("8Wf5TiAheLUqBrKXeYg2JtAFFMWtKdG2BSFgqUcPVwTt");
pub const PUMPFUN_FEE_PROGRAM: Pubkey = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");
pub const PUMPFUN_GLOBAL: Pubkey = pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");
pub const PUMP_FUN_EVENT_AUTHORITY: Pubkey =
    pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1");
pub const PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR: Pubkey =
    pubkey!("Hq2wp8uJ9jCPsYgNHex8RtqdvMPfVGoYwjvF1ATiwn2Y");

//pumpswap accounts
pub const PUMPSWAP_PROGRAM_ID: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub const PUMPFUN_MIGRATE: Pubkey = pubkey!("39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg");
pub const PUMPSWAP_GLOBAL: Pubkey = pubkey!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");
pub const PUMPSWAP_FEE_1: Pubkey = pubkey!("62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
pub const PUMPSWAP_EVENT_AUTH: Pubkey = pubkey!("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR");
pub const PUMPSWAP_FEE_CONFIG: Pubkey = pubkey!("5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");
pub const PUMPSWAP_FEE_PROGRAM: Pubkey = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const PUMPSWAP_GLOBAL_VOLUME_ACCUMULATOR: Pubkey =
    pubkey!("C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw");

//mayhem
pub const MAYHEM_PROTOCOL_FEE_RECIPIENT: Pubkey =
    pubkey!("GesfTA3X2arioaHp8bbKdjG9vJtskViWACZoYvxp4twS");

//Zero slot tip accounts
pub const ZERO_SLOT_TIP_ACCOUNTS: [Pubkey; 21] = [
    pubkey!("6fQaVhYZA4w3MBSXjJ81Vf6W1EDYeUPXpgVQ6UQyU1Av"), // 0slot_dot_trade.sol
    pubkey!("4HiwLEP2Bzqj3hM2ENxJuzhcPCdsafwiet3oGkMkuQY4"), // 0slot_dot_trade_tip15.sol
    pubkey!("7toBU3inhmrARGngC7z6SjyP85HgGMmCTEwGNRAcYnEK"), // 0slot_dot_trade_tip16.sol
    pubkey!("8mR3wB1nh4D6J9RUCugxUpc6ya8w38LPxZ3ZjcBhgzws"), // 0slot_dot_trade_tip17.sol
    pubkey!("6SiVU5WEwqfFapRuYCndomztEwDjvS5xgtEof3PLEGm9"), // 0slot_dot_trade_tip18.sol
    pubkey!("TpdxgNJBWZRL8UXF5mrEsyWxDWx9HQexA9P1eTWQ42p"),  // 0slot_dot_trade_tip19.sol
    pubkey!("D8f3WkQu6dCF33cZxuAsrKHrGsqGP2yvAHf8mX6RXnwf"), // 0slot_dot_trade_tip20.sol
    pubkey!("GQPFicsy3P3NXxB5piJohoxACqTvWE9fKpLgdsMduoHE"), // 0slot_dot_trade_tip21.sol
    pubkey!("Ey2JEr8hDkgN8qKJGrLf2yFjRhW7rab99HVxwi5rcvJE"), // 0slot_dot_trade_tip22.sol
    pubkey!("4iUgjMT8q2hNZnLuhpqZ1QtiV8deFPy2ajvvjEpKKgsS"), // 0slot_dot_trade_tip23.sol
    pubkey!("3Rz8uD83QsU8wKvZbgWAPvCNDU6Fy8TSZTMcPm3RB6zt"), // 0slot_dot_trade_tip24.sol
    pubkey!("DiTmWENJsHQdawVUUKnUXkconcpW4Jv52TnMWhkncF6t"), // 0slot_dot_trade_tip1.sol
    pubkey!("HRyRhQ86t3H4aAtgvHVpUJmw64BDrb61gRiKcdKUXs5c"), // 0slot_dot_trade_tip2.sol
    pubkey!("7y4whZmw388w1ggjToDLSBLv47drw5SUXcLk6jtmwixd"), // 0slot_dot_trade_tip6.sol
    pubkey!("J9BMEWFbCBEjtQ1fG5Lo9kouX1HfrKQxeUxetwXrifBw"), // 0slot_dot_trade_tip7.sol
    pubkey!("8U1JPQh3mVQ4F5jwRdFTBzvNRQaYFQppHQYoH38DJGSQ"), // 0slot_dot_trade_tip8.sol
    pubkey!("Eb2KpSC8uMt9GmzyAEm5Eb1AAAgTjRaXWFjKyFXHZxF3"), // 0slot_dot_trade_tip3.sol
    pubkey!("FCjUJZ1qozm1e8romw216qyfQMaaWKxWsuySnumVCCNe"), // 0slot_dot_trade_tip5.sol
    pubkey!("ENxTEjSQ1YabmUpXAdCgevnHQ9MHdLv8tzFiuiYJqa13"), // 0slot_dot_trade_tip9.sol
    pubkey!("6rYLG55Q9RpsPGvqdPNJs4z5WTxJVatMB8zV3WJhs5EK"), // 0slot_dot_trade_tip10.sol
    pubkey!("Cix2bHfqPcKcM233mzxbLk14kSggUUiz2A87fJtGivXr"), // 0slot_dot_trade_tip12.sol
];

//Helius tip accounts
pub const HELIUS_TIP_ACCOUNTS: [Pubkey; 10] = [
    pubkey!("4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE"),
    pubkey!("D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ"),
    pubkey!("9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta"),
    pubkey!("5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn"),
    pubkey!("2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD"),
    pubkey!("2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ"),
    pubkey!("wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF"),
    pubkey!("3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT"),
    pubkey!("4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey"),
    pubkey!("4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or"),
];
