////Compute budget program discriminator
pub const SET_BUDGET_COMPUTE_UNIT_LIMIT_DISCRIMINATOR: [u8; 1] = [2];
pub const SET_BUDGET_COMPUTE_UNIT_PRICE_DISCRIMINATOR: [u8; 1] = [3];

//////Pumpswap discriminator
///
///
pub static CREATE_POOL_DISCRIMINATOR: [u8; 8] = [233, 146, 209, 142, 207, 104, 64, 188];
pub static CREATE_POOL_EVENT_DISCRIMINATOR: [u8; 8] = [177, 49, 12, 210, 160, 118, 167, 116];
pub static PUMPSWAP_SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
pub static PUMPSWAP_BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
pub static BUY_EXACT_QUOTE_IN_DISCRIMINATOR: [u8; 8] = [198, 46, 21, 82, 180, 217, 232, 112];
pub static MIGRATE_DISCRIMINATOR: [u8; 8] = [155, 234, 231, 146, 236, 158, 162, 30];
pub static EVENT_AUTH_ACC_DISC: [u8; 8] = [228, 69, 165, 46, 81, 203, 154, 29];

pub static BUY_EVENT_DISC: [u8; 8] = [103, 244, 82, 31, 44, 245, 119, 119];
pub static SELL_EVENT_DISC: [u8; 8] = [62, 47, 55, 10, 165, 3, 220, 42];