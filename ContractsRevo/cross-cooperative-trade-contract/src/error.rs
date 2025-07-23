use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminError {
    AlreadyInitialized = 1,
    UnauthorizedAccess = 2,
    NotInitialized = 3,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeError {
    TradeOfferNotFound = 1,
    InvalidTradeStatus = 2,
    UnauthorizedAccess = 3,
    TradeExpired = 4,
    CannotAcceptOwnOffer = 5,
    InvalidQuantity = 6,
    BarterAgreementNotFound = 7,
}
