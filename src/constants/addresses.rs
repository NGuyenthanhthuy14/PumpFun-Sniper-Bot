use once_cell::sync::Lazy;
use solana_sdk::{pubkey, pubkey::Pubkey};

pub const BUDGET_COMPUTE_PROGRAM: Pubkey = pubkey!("ComputeBudget111111111111111111111111111111");
pub const PUMPFUN_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const PUMPFUN_FEE_RECIPIENT: Pubkey = pubkey!("62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
pub const PUMPFUN_FEE_CONFIG: Pubkey = pubkey!("8Wf5TiAheLUqBrKXeYg2JtAFFMWtKdG2BSFgqUcPVwTt");
pub const PUMPFUN_FEE_PROGRAM: Pubkey = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const PUMPFUN_GLOBAL_VOLUME_ACCUMULATOR: Pubkey =
    pubkey!("Hq2wp8uJ9jCPsYgNHex8RtqdvMPfVGoYwjvF1ATiwn2Y");
pub const PUMPSWAP_PROGRAM: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");
pub const PUMPFUN_GLOBAL: Pubkey = pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");
pub const ASSOCIATED_PROGRAM: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const PUMP_FUN_EVENT_AUTHORITY: Pubkey =
    pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1");
pub const MAYHEM_FEE_RECIPIENT: Pubkey = pubkey!("GesfTA3X2arioaHp8bbKdjG9vJtskViWACZoYvxp4twS");

pub static CREATOR_VAULT_SEED: &[u8] = b"creator-vault";
pub static USER_VOLUME_ACCUMULATOR_SEED: &[u8] = b"user_volume_accumulator";
pub static PUMPFUN_BONDING_CURVE: &[u8] = b"bonding-curve";

pub const TOKEN_2022_PROGRAM: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/////////////////////////Pro traders /////////////////////////

pub struct Trader {
    pub name: &'static str,
    pub address: Pubkey,
}

pub static PRO_TRADERS: Lazy<Vec<Trader>> = Lazy::new(|| {
    vec![
        Trader {
            name: "YOLO",
            address: pubkey!("Av3xWHJ5EsoLZag6pr7LKbrGgLRTaykXomDD5kBhL9YQ"),
        },
        Trader {
            name: "KEV",
            address: pubkey!("BTf4A2exGK9BCVDNzy65b9dUzXgMqB4weVkvTMFQsadd"),
        },
        Trader {
            name: "HENN",
            address: pubkey!("FRbUNvGxYNC1eFngpn7AD3f14aKKTJVC6zSMtvj2dyCS"),
        },
        Trader {
            name: "CAP",
            address: pubkey!("CAPn1yH4oSywsxGU456jfgTrSSUidf9jgeAnHceNUJdw"),
        },
        Trader {
            name: "BANDIT",
            address: pubkey!("5B79fMkcFeRTiwm7ehsZsFiKsC7m7n1Bgv9yLxPp9q2X"),
        },
        Trader {
            name: "BASTILE",
            address: pubkey!("3kebnKw7cPdSkLRfiMEALyZJGZ4wdiSRvmoN4rD1yPzV"),
        },
        Trader {
            name: "QTDEGEN",
            address: pubkey!("7tiRXPM4wwBMRMYzmywRAE6jveS3gDbNyxgRrEoU6RLA"),
        },
        Trader {
            name: "ART",
            address: pubkey!("CgaA9a1JwAXJyfHuvZ7VW8YfTVRkdiT5mjBBSKcg7Rz5"),
        },
        Trader {
            name: "LECK",
            address: pubkey!("98T65wcMEjoNLDTJszBHGZEX75QRe8QaANXokv4yw3Mp"),
        },
        Trader {
            name: "CENTED",
            address: pubkey!("98T65wcMEjoNLDTJszBHGZEX75QRe8QaANXokv4yw3Mp"),
        },
        Trader {
            name: "Gh0stee",
            address: pubkey!("2kv8X2a9bxnBM8NKLc6BBTX2z13GFNRL4oRotMUJRva9"),
        },
        Trader {
            name: "Trey",
            address: pubkey!("831yhv67QpKqLBJjbmw2xoDUeeFHGUx8RnuRj9imeoEs")
        },
        Trader {
            name: "Log",
            address: pubkey!("2pUUZYtokRgDV2YzL6M5pjb1jyoHE367yU1sdQ7ac3ea")
        },
        Trader {
            name: "Til",
            address: pubkey!("EHg5YkU2SZBTvuT87rUsvxArGp3HLeye1fXaSDfuMyaf")
        },
        Trader {
            name: "EustazZ",
            address: pubkey!("FqamE7xrahg7FEWoByrx1o8SeyHt44rpmE6ZQfT7zrve")
        },
        Trader {
            name: "Voyage",
            address: pubkey!("BJeWdzF9HVeWHktxwS9u8FbCrdSDRrDoS3S9stLafdQJ")
        }
    ]
});

impl Trader {
    pub fn find_by_address(address: Pubkey) -> Option<&'static str> {
        PRO_TRADERS.iter().find_map(|trader| {
            if trader.address == address {
                Some(trader.name)
            } else {
                None
            }
        })
    }
}
