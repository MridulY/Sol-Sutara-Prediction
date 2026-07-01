use anchor_lang::prelude::*;

pub mod amm;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod merkle;
pub mod state;

use instructions::*;

declare_id!("SutaraProtoco1111111111111111111111111111111");

#[program]
pub mod sutara_protocol {
    use super::*;

    // ─── Protocol Admin ───────────────────────────────────────────────────────

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        params: initialize_protocol::InitializeProtocolParams,
    ) -> Result<()> {
        initialize_protocol::handler(ctx, params)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        pause::handler(ctx)
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        unpause::handler(ctx)
    }

    pub fn update_fees(
        ctx: Context<UpdateFees>,
        params: update_fees::UpdateFeesParams,
    ) -> Result<()> {
        update_fees::handler(ctx, params)
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        params: update_config::UpdateConfigParams,
    ) -> Result<()> {
        update_config::handler(ctx, params)
    }

    // ─── Market Lifecycle ─────────────────────────────────────────────────────

    pub fn create_market(
        ctx: Context<CreateMarket>,
        params: create_market::CreateMarketParams,
    ) -> Result<()> {
        create_market::handler(ctx, params)
    }

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        params: initialize_pool::InitializePoolParams,
    ) -> Result<()> {
        initialize_pool::handler(ctx, params)
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        close_market::handler(ctx)
    }

    // ─── Liquidity ────────────────────────────────────────────────────────────

    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        params: deposit_liquidity::DepositLiquidityParams,
    ) -> Result<()> {
        deposit_liquidity::handler(ctx, params)
    }

    pub fn withdraw_liquidity(
        ctx: Context<WithdrawLiquidity>,
        params: withdraw_liquidity::WithdrawLiquidityParams,
    ) -> Result<()> {
        withdraw_liquidity::handler(ctx, params)
    }

    pub fn claim_lp_fees(ctx: Context<ClaimLpFees>) -> Result<()> {
        claim_lp_fees::handler(ctx)
    }

    // ─── Trading ──────────────────────────────────────────────────────────────

    pub fn buy_shares(
        ctx: Context<BuyShares>,
        params: buy_shares::BuySharesParams,
    ) -> Result<()> {
        buy_shares::handler(ctx, params)
    }

    pub fn sell_shares(
        ctx: Context<SellShares>,
        params: sell_shares::SellSharesParams,
    ) -> Result<()> {
        sell_shares::handler(ctx, params)
    }

    // ─── Settlement ───────────────────────────────────────────────────────────

    pub fn submit_proof(
        ctx: Context<SubmitProof>,
        params: submit_proof::SubmitProofParams,
    ) -> Result<()> {
        submit_proof::handler(ctx, params)
    }

    pub fn validate_result(ctx: Context<ValidateResult>) -> Result<()> {
        validate_result::handler(ctx)
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>) -> Result<()> {
        resolve_market::handler(ctx)
    }

    // ─── Claims ───────────────────────────────────────────────────────────────

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        claim_rewards::handler(ctx)
    }

    // ─── Emergency ────────────────────────────────────────────────────────────

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        emergency_withdraw::handler(ctx)
    }
}
