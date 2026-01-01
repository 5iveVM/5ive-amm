import social_types;
import market_logic;
import voting_logic;

pub fn create_market(
    market: Market @mut @init,
    creator: account @signer,
    question: string,
    duration_seconds: u64
) {
    market_logic::create_market(market, creator, question, duration_seconds);
}

pub fn resolve_market(
    market: Market @mut,
    resolver: account @signer,
    outcome_yes: bool
) {
    market_logic::resolve_market(market, resolver, outcome_yes);
}

pub fn init_vote(
    vote: Vote @mut @init,
    market: Market,
    owner: account @signer
) {
    voting_logic::init_vote(vote, market, owner);
}

pub fn vote_yes(
    market: Market @mut,
    vote: Vote @mut,
    voter: account @signer,
    amount: u64
) {
    voting_logic::vote_yes(market, vote, voter, amount);
}

pub fn vote_no(
    market: Market @mut,
    vote: Vote @mut,
    voter: account @signer,
    amount: u64
) {
    voting_logic::vote_no(market, vote, voter, amount);
}

pub fn claim_winnings(
    market: Market @mut,
    vote: Vote @mut,
    user: account @signer
) -> u64 {
    return voting_logic::claim_winnings(market, vote, user);
}
