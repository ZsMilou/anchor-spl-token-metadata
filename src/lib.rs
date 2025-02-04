pub mod anchor_token_metadata {
    use anchor_lang::prelude::*;
    use anchor_lang::solana_program::{
        self,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar,
    };
    use spl_token_metadata::{
        instruction::{update_metadata_accounts, CreateMetadataAccountArgs, MetadataInstruction},
        state::{Creator, Data},
    };

    pub type Result<T> = std::result::Result<T, error::Error>;

    pub fn create_metadata<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, CreateMetadata<'info>>,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
    ) -> Result<()> {
        let ix = create_metadata_ix(
            *ctx.accounts.token_metadata_program.key,
            *ctx.accounts.metadata.key,
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.update_authority.key(),
            name,
            symbol,
            uri,
            creators,
            seller_fee_basis_points,
            update_authority_is_signer,
            is_mutable,
        );
        solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.metadata.clone(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.update_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            ctx.signer_seeds,
        )?;
        Ok(())
    }
    pub fn update_metadata<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, UpdateMetadataAccount<'info>>,
        new_update_authority: Option<Pubkey>,
        data: Option<Data>,
        primary_sale_happened: Option<bool>,
    ) -> Result<()> {
        let ix = update_metadata_accounts(
            ctx.accounts.token_metadata_program.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.update_authority.key(),
            new_update_authority,
            data,
            primary_sale_happened,
        );
        solana_program::program::invoke_signed(
            &ix,
            &[ctx.accounts.metadata, ctx.accounts.update_authority],
            ctx.signer_seeds,
        )?;
        Ok(())
    }

    #[derive(Clone)]
    pub struct TokenMetadata;
    impl anchor_lang::AccountDeserialize for TokenMetadata {
        fn try_deserialize(buf: &mut &[u8]) -> Result<Self> {
            TokenMetadata::try_deserialize_unchecked(buf)
        }
        fn try_deserialize_unchecked(_buf: &mut &[u8]) -> Result<Self> {
            Ok(TokenMetadata)
        }
    }
    impl anchor_lang::Owner for TokenMetadata {
        fn owner() -> Pubkey {
            spl_token_metadata::ID
        }
    }
    impl anchor_lang::Id for TokenMetadata {
        fn id() -> Pubkey {
            spl_token_metadata::id()
        }
    }
    //don't need to use anchor macros here because the token metadata program does the checks
    #[derive(Accounts)]
    pub struct CreateMetadata<'info> {
        pub metadata: AccountInfo<'info>, //Metadata key (pda of ['metadata', program id, mint id])
        pub mint: AccountInfo<'info>,     //mint of the token we are creating metadata for
        pub mint_authority: AccountInfo<'info>,
        pub payer: AccountInfo<'info>,
        pub update_authority: AccountInfo<'info>, //this is the account that will have future ability to update the newly created metadata
        #[account(address = spl_token_metadata::id())]
        pub token_metadata_program: AccountInfo<'info>,
        pub system_program: Program<'info, System>,
        pub rent: Sysvar<'info, Rent>,
    }
    #[derive(Accounts)]
    pub struct UpdateMetadataAccount<'info> {
        #[account(mut)]
        pub metadata: AccountInfo<'info>,
        pub update_authority: AccountInfo<'info>,
        #[account(address = spl_token_metadata::id())]
        pub token_metadata_program: AccountInfo<'info>,
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_metadata_ix(
        program_id: Pubkey,
        metadata_account: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        payer: Pubkey,
        update_authority: Pubkey,
        name: String,
        symbol: String,
        uri: String,
        creators: Option<Vec<Creator>>,
        seller_fee_basis_points: u16,
        update_authority_is_signer: bool,
        is_mutable: bool,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(metadata_account, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(mint_authority, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(update_authority, update_authority_is_signer),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
            data: MetadataInstruction::CreateMetadataAccount(CreateMetadataAccountArgs {
                data: Data {
                    name,
                    symbol,
                    uri,
                    seller_fee_basis_points,
                    creators,
                },
                is_mutable,
            })
            .try_to_vec()
            .unwrap(),
        }
    }
}
