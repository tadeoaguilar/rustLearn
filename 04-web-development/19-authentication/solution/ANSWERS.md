# Answers · 19 Authentication

## Exercise 3: Bob was promoted but his token still says `user`. Why, and what are the options?

The role is a claim *inside* the signed access token, and the server trusts
the token without looking anything up — that's the point of a JWT. It stays
valid, with the old role, until it expires. Options:

1. **Accept it**: access tokens live 15 minutes; Bob gets the new role at his
   next refresh (or logs in again). Fine for promotions.
2. **Look the role up per request** for sensitive checks (the token proves
   identity; the database decides permissions). One query, always current.
3. **Revoke** his tokens on role changes (needs a deny-list or a per-user
   "tokens issued before X are invalid" timestamp). Essential for *demotions*
   and account bans, where 15 minutes of leftover access is unacceptable.

## Exercise 4: The cookie is `SameSite=Lax`. Why still use CSRF tokens?

`SameSite=Lax` stops the cookie on cross-site POSTs in modern browsers, which
blocks most CSRF. But it's a browser policy, not a server guarantee: older
browsers ignore it; it doesn't cover *same-site* attackers (a compromised or
user-content subdomain, `evil.example.com` attacking `app.example.com`);
top-level GET navigations still carry the cookie, so any state-changing GET is
exposed; and a misconfigured `SameSite=None` reopens everything. CSRF tokens
are defence in depth that the server itself verifies.

## Exercise 5: Why is a fast hash fine for API keys?

Slow hashes exist to make *guessing* expensive, and they're needed because
human passwords have little entropy — an attacker with the hash can try the
likely ones. An API key is 192 random bits; there's no list of likely keys,
and 2^192 guesses is out of reach at any speed. A fast hash still prevents
using a leaked database directly, and lets us find a key with one lookup by
its hash — impossible with salted Argon2 hashes.
