/// Type-safe chain identifier with convenient constructors
mod chain;
/// Type-safe referral code
mod referral;
/// Type-safe slippage percentage with validation
mod slippage;

pub use chain::Chain;
pub use referral::ReferralCode;
pub use slippage::Slippage;
