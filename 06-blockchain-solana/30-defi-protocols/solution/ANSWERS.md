# Answers · 30 DeFi Protocols

## Exercise 1: Why lock `MINIMUM_LIQUIDITY`?

The *first-depositor inflation attack*. Without it, an attacker deposits
1 unit of each token and gets 1 share, then *donates* a large amount
straight to the vault (or, here, to the reserves through a skewed deposit).
One share is now worth, say, 1,000,000 tokens. The next depositor's
`amount * supply / reserve` rounds down to 0 shares -- their deposit is
absorbed and the attacker's single share owns it. Burning the first 1,000
shares means the share price starts tiny and can only be inflated by
donating 1,000 times as much, which the attacker loses. Uniswap V2 sends
them to the zero address; this pool just never mints them.

## Exercise 2: Why store the reserves?

Anyone can transfer tokens into a vault. If prices came from vault
balances, a donation would move the price -- harmless for an AMM's own
swaps (the donor loses money), but anything that *reads* the price (a
lending market valuing LP tokens, an oracle built on the pool) could be
manipulated within one transaction. Storing reserves means only the pool's
own instructions change them, and unsolicited tokens are simply ignored.
(Production AMMs also track a time-weighted average price, so a single
block's price can't be used against anyone.)

## Exercise 3: Why the kink?

Lenders must be able to withdraw. At 100% utilization every token is lent
out and nobody can leave. Past the kink the rate climbs steeply -- 62% at
90% utilization in the tests' model -- which makes borrowers repay and
attracts new lenders, pulling utilization back down. Below the kink, rates
stay low to keep borrowing attractive.

## Exercise 5

**LTV versus threshold.** The gap is a buffer. A borrow is only allowed up
to the LTV (75%); liquidation starts at the threshold (80%). Without a gap,
a position opened at the limit would be liquidatable by the next price
tick, and borrowers would lose the bonus to liquidators for nothing. With
it, the price must fall about 6% before a maximal borrower can be liquidated.

**Any price feed.** The attacker creates their own feed -- the oracle
program lets anyone create one -- reporting their collateral at $1,000,000,
borrows the whole liquidity vault against a little collateral, and walks
away. The market must check that the feed accounts are *the ones it was
configured with* (and owned by the oracle program, and fresh). The tests
try exactly this.
