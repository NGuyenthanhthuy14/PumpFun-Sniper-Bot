/// Identify a System Program instruction by its u32 LE discriminator.
pub fn identify_system_program_ix(data: &[u8]) -> String {
    let disc = data
        .get(..4)
        .and_then(|b| <[u8; 4]>::try_from(b).ok())
        .map(u32::from_le_bytes);
    match disc {
        Some(0) => "SystemProgram:CreateAccount".to_string(),
        Some(1) => "SystemProgram:Assign".to_string(),
        Some(2) => "SystemProgram:Transfer".to_string(),
        Some(3) => "SystemProgram:CreateAccountWithSeed".to_string(),
        Some(4) => "SystemProgram:AdvanceNonceAccount".to_string(),
        Some(5) => "SystemProgram:WithdrawNonceAccount".to_string(),
        Some(6) => "SystemProgram:InitializeNonceAccount".to_string(),
        Some(7) => "SystemProgram:AuthorizeNonceAccount".to_string(),
        Some(8) => "SystemProgram:Allocate".to_string(),
        Some(9) => "SystemProgram:AllocateWithSeed".to_string(),
        Some(10) => "SystemProgram:AssignWithSeed".to_string(),
        Some(11) => "SystemProgram:TransferWithSeed".to_string(),
        Some(12) => "SystemProgram:UpgradeNonceAccount".to_string(),
        _ => "SystemProgram:Unknown".to_string(),
    }
}

/// Identify an Associated Token Program instruction by its single-byte discriminator.
pub fn identify_ata_program_ix(data: &[u8]) -> String {
    match data.first() {
        Some(&0) | None => "AssociatedTokenProgram:Create".to_string(),
        Some(&1) => "AssociatedTokenProgram:CreateIdempotent".to_string(),
        Some(&2) => "AssociatedTokenProgram:RecoverNested".to_string(),
        _ => "AssociatedTokenProgram:Unknown".to_string(),
    }
}

/// Identify a Token-2022 Program instruction by its single-byte discriminator.
pub fn identify_token2022_program_ix(data: &[u8]) -> String {
    match data.first() {
        Some(&0) => "Token2022:InitializeMint".to_string(),
        Some(&1) => "Token2022:InitializeAccount".to_string(),
        Some(&2) => "Token2022:InitializeMultisig".to_string(),
        Some(&3) => "Token2022:Transfer".to_string(),
        Some(&4) => "Token2022:Approve".to_string(),
        Some(&5) => "Token2022:Revoke".to_string(),
        Some(&6) => "Token2022:SetAuthority".to_string(),
        Some(&7) => "Token2022:MintTo".to_string(),
        Some(&8) => "Token2022:Burn".to_string(),
        Some(&9) => "Token2022:CloseAccount".to_string(),
        Some(&10) => "Token2022:FreezeAccount".to_string(),
        Some(&11) => "Token2022:ThawAccount".to_string(),
        Some(&12) => "Token2022:TransferChecked".to_string(),
        Some(&13) => "Token2022:ApproveChecked".to_string(),
        Some(&14) => "Token2022:MintToChecked".to_string(),
        Some(&15) => "Token2022:BurnChecked".to_string(),
        Some(&16) => "Token2022:InitializeAccount2".to_string(),
        Some(&17) => "Token2022:SyncNative".to_string(),
        Some(&18) => "Token2022:InitializeAccount3".to_string(),
        Some(&20) => "Token2022:InitializeMint2".to_string(),
        Some(&25) => "Token2022:InitializeImmutableOwner".to_string(),
        _ => "Token2022:Unknown".to_string(),
    }
}
