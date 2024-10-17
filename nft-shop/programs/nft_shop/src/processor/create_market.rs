use crate::{
    error::ErrorCode,
    state::{MarketState, SellingResourceState, MINIMUM_BALANCE_FOR_SYSTEM_ACCS},
    utils::*,
    CreateMarket,
};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::token::accessor;

impl<'info> CreateMarket<'info> {
    pub fn process(
        &mut self,
        _treasury_owner_bump: u8,
        name: String,
        description: String,
        mutable: bool,
        price: u64,
        pieces_in_one_wallet: Option<u64>,
        start_date: u64,
        end_date: Option<u64>,
    ) -> Result<()> {
        let market = &mut self.market;
        let store = &self.store;
        let selling_resource_owner = &self.selling_resource_owner;
        let selling_resource = &mut self.selling_resource;
        let treasury_mint = self.treasury_mint.to_account_info();
        let treasury_holder = self.treasury_holder.to_account_info();
        let treasury_owner = &self.treasury_owner;

        if name.len() > NAME_MAX_LEN {
            return Err(ErrorCode::NameIsTooLong.into());
        }

        if description.len() > DESCRIPTION_MAX_LEN {
            return Err(ErrorCode::DescriptionIsTooLong.into());
        }

        // Pieces in one wallet cannot be greater than Max Supply value
        if pieces_in_one_wallet.is_some()
            && selling_resource.max_supply.is_some()
            && pieces_in_one_wallet.unwrap() > selling_resource.max_supply.unwrap()
        {
            return Err(ErrorCode::PiecesInOneWalletIsTooMuch.into());
        }

        // Only new just created selling resource can be used to create market
        if selling_resource.state != SellingResourceState::Created {
            return Err(ErrorCode::SellingResourceAlreadyTaken.into());
        }

        // start_date cannot be in the past
        if start_date < Clock::get().unwrap().unix_timestamp as u64 {
            return Err(ErrorCode::StartDateIsInPast.into());
        }

        // end_date should not be greater than start_date
        if end_date.is_some() && start_date > end_date.unwrap() {
            return Err(ErrorCode::EndDateIsEarlierThanBeginDate.into());
        }

        let is_native = treasury_mint.key() == System::id();

        if !is_native {
            if treasury_mint.owner != &anchor_spl::token::ID
                || treasury_holder.owner != &anchor_spl::token::ID
            {
                return Err(ProgramError::IllegalOwner.into());
            }

            if accessor::mint(&treasury_holder)? != *treasury_mint.key {
                return Err(ProgramError::InvalidAccountData.into());
            }

            if accessor::authority(&treasury_holder)? != treasury_owner.key() {
                return Err(ProgramError::InvalidAccountData.into());
            }
        } else {
            // for native SOL we use PDA as a treasury holder
            // because of security reasons(only program can spend this SOL)
            if treasury_holder.key != treasury_owner.key {
                return Err(ProgramError::InvalidAccountData.into());
            }

            // we need fund treasury holder account such as it will hold some metadata with SOL balance
            invoke(
                &system_instruction::transfer(
                    &selling_resource_owner.key(),
                    &treasury_holder.key(),
                    MINIMUM_BALANCE_FOR_SYSTEM_ACCS,
                ),
                &[
                    selling_resource_owner.to_account_info(),
                    treasury_holder.to_account_info(),
                ],
            )?;
        }

        // Check selling resource ownership
        assert_keys_equal(selling_resource.owner, selling_resource_owner.key())?;

        market.store = store.key();
        market.selling_resource = selling_resource.key();
        market.treasury_mint = treasury_mint.key();
        market.treasury_holder = treasury_holder.key();
        market.treasury_owner = treasury_owner.key();
        market.owner = selling_resource_owner.key();
        market.name = puffed_out_string(name, NAME_MAX_LEN);
        market.description = puffed_out_string(description, DESCRIPTION_MAX_LEN);
        market.mutable = mutable;
        market.price = price;
        market.pieces_in_one_wallet = pieces_in_one_wallet;
        market.start_date = start_date;
        market.end_date = end_date;
        market.state = MarketState::Created;
        selling_resource.state = SellingResourceState::InUse;

        Ok(())
    }
}
