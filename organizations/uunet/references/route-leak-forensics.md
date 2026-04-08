# Route Leak Forensics — Deep Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌‌‌​​‍​​​‌‌​​‌‍​​‌​​‌​​‍​​​‌​​‌​‍​​​​‌​​​‍‌​​​​​‌‌⁠‍⁠

Load this file ONLY when investigating an active or historical BGP route leak, hijack, or large-scale propagation incident.

## The AS7007 timeline (25 April 1997)

This is the canonical case study every backbone engineer must internalize. The post-mortem from MAI Network Services' director (Vincent Bono on NANOG, 26 Apr 1997) is the primary source.

| Time (EST) | Event |
|---|---|
| 11:30 | MAI's border router AS7007 receives a 23,000-route view from a downstream ISP. **No inbound distribute-list was imposed on the downstream.** |
| 11:30+ | A router bug deaggregates the routes to /24 prefixes (more specific than anything legitimate on the internet) and **strips the AS-path**, replacing it with just `7007`. |
| 11:30+ | The /24s win BGP best-path selection on every router that hears them, because longest-prefix match beats AS-path length. |
| 11:45 | MAI's MAE-East border (AS6082) propagates 73,000+ routes; their internal monitor (set to alarm at >45k routes) trips. They tear down the BGP session with AS1790. |
| 12:00 | Other ISPs are still calling MAI saying they see the routes. MAI reboots the 7007 router. |
| 12:00+ | The router comes back up and re-announces the full view to AS1790 — this time normally, with proper AS-paths. |
| 12:15 | Panic. MAI **physically unplugs all peering**. |
| 12:25 | Sprint NOC calls: "we are about to turn down the DS-3 to your 7007 router because we are still seeing your routes." MAI confirms the router has no power. |
| 14:14 | Sprint techs report the 7007 routes "just keep appearing again" in their BGP tables. |
| ~16:45 | MAI is still fielding calls from ISPs worldwide. |
| ~19:00 | Last residual routes finally clear MSN's west-coast switched backbone. |

**Read the lessons carefully — the obvious one is wrong.**

The naive lesson is "filter your peers." Everyone already knew that in 1997. The real lessons are subtler:

1. **Filter every neighbor including customers.** The route leak entered MAI from a *downstream customer*, not a peer. MAI's filter logic was "we filter peers, customers are trusted." That assumption blew up the internet for a day.
2. **Filters at the second hop do not save you.** Sprint had filters; the routes still propagated through their network because once a /24 enters BGP, it wins on longest-prefix match regardless of what you would have filtered at *your* edge.
3. **BGP has no "withdraw" guarantee.** Sprint kept seeing the routes after MAI was physically powered off. BGP relies on hop-by-hop withdraw messages, and any router along the way that drops a withdraw (or had its session reset) will retain the stale route until something else displaces it.
4. **`maxas-limit` would not have helped here** — the AS-path was stripped to a single AS. **`maximum-prefix` would have helped enormously**: 73,000+ prefixes from a /23 customer would have tripped any sane limit.
5. **Out-of-band coordination matters more than the routers.** The actual containment was phone calls between NOCs, not BGP magic.

## The canonical filter stack (apply to EVERY neighbor)

```
neighbor 192.0.2.1 remote-as 64500
neighbor 192.0.2.1 description CUSTOMER-XYZ
neighbor 192.0.2.1 password <shared-secret>
neighbor 192.0.2.1 ttl-security hops 1               ! GTSM, RFC 5082
neighbor 192.0.2.1 prefix-list cust-xyz-in in        ! HARD prefix list
neighbor 192.0.2.1 filter-list 10 in                 ! AS-path filter: ^64500$
neighbor 192.0.2.1 maximum-prefix 50 80 restart 30   ! warn at 80%, tear down at 50, restart in 30 min
neighbor 192.0.2.1 maxas-limit 20                    ! reject AS-paths >20 hops
neighbor 192.0.2.1 send-community
neighbor 192.0.2.1 route-map cust-xyz-in in          ! set local-pref 200, tag with community
```

Each line addresses a different historical failure:
- `password` — defeats off-path TCP RST attacks against eBGP
- `ttl-security hops 1` — GTSM, drops eBGP packets with TTL≠255, defeats remote injection
- `prefix-list` — the AS7007 fix; only allow what the customer is allocated
- `filter-list` (AS-path) — rejects leaked transit; only allow paths originating in the customer's own AS
- `maximum-prefix` — the AS7007 *containment* fix; tear down before melting the RIB
- `maxas-limit` — defeats the prepending-loop bugs that plagued early implementations
- `route-map` — sets LOCAL_PREF and tags with a community so the rest of your policy is consistent

## Bogon and martian filtering (inbound from any neighbor)

You must drop the following at every eBGP edge:

```
ip prefix-list bogons-v4 deny 0.0.0.0/8 le 32         ! "this network"
ip prefix-list bogons-v4 deny 10.0.0.0/8 le 32        ! RFC 1918
ip prefix-list bogons-v4 deny 100.64.0.0/10 le 32     ! RFC 6598 CGN
ip prefix-list bogons-v4 deny 127.0.0.0/8 le 32       ! loopback
ip prefix-list bogons-v4 deny 169.254.0.0/16 le 32    ! link-local
ip prefix-list bogons-v4 deny 172.16.0.0/12 le 32     ! RFC 1918
ip prefix-list bogons-v4 deny 192.0.0.0/24 le 32      ! IETF protocol
ip prefix-list bogons-v4 deny 192.0.2.0/24 le 32      ! TEST-NET-1
ip prefix-list bogons-v4 deny 192.168.0.0/16 le 32    ! RFC 1918
ip prefix-list bogons-v4 deny 198.18.0.0/15 le 32     ! benchmarking
ip prefix-list bogons-v4 deny 198.51.100.0/24 le 32   ! TEST-NET-2
ip prefix-list bogons-v4 deny 203.0.113.0/24 le 32    ! TEST-NET-3
ip prefix-list bogons-v4 deny 224.0.0.0/3 le 32       ! multicast + reserved
ip prefix-list bogons-v4 deny 0.0.0.0/0 ge 25         ! anything more specific than /24
ip prefix-list bogons-v4 permit 0.0.0.0/0 le 24
```

The last two lines matter: **never accept anything more specific than /24 from a peer or transit** — that is the AS7007 signature. (Some networks accept /25 from select peers; document the exception.)

## RPKI / MANRS posture

The modern (post-2018) state of the art is RPKI Origin Validation:

- Sign your own prefixes with a ROA (Route Origin Authorization) at your RIR.
- Validate received routes against the RPKI cache (`rtr-server`).
- Set policy: **invalid → drop, unknown → low local-pref, valid → high local-pref**. Cloudflare, NTT, AT&T, and most tier-1s now drop RPKI-invalid routes outright.
- Join MANRS (Mutually Agreed Norms for Routing Security) and implement all four actions: filtering, anti-spoofing, coordination, and global validation.

What RPKI does NOT do: it does not protect against AS-path manipulation or route leaks where the origin is correct but the path is forged. For that, you need BGPsec (still not deployed at scale) or ASPA (in IETF draft).

## Distinguishing leak from hijack

- **Leak**: an AS announces a prefix it should not, usually because of misconfiguration. The prefix is real, the origin is the legitimate owner upstream, the AS-path is implausible. Pattern: customer learns transit routes and re-announces them to its other transit. Detection: an unexpected AS appears in the path.
- **Hijack**: an AS originates a prefix it does not own. The origin is wrong. Pattern: a state actor or criminal announces /24 cuts of a victim's /16 to attract traffic. Detection: an unexpected AS appears as the origin.
- **Sub-prefix hijack**: like a hijack but using a more-specific. Always wins. Mitigation: announce your own /24 cuts so attackers cannot get more-specific than you.

When you investigate an incident, the first question is "what is the origin AS for this prefix in the global table right now?" Use a looking glass like RIPE RIS, RouteViews, or `bgp.he.net`.

## RTBH and Flowspec for active mitigation

For an in-flight DDoS that exceeds your line-card filter capacity:

- **Destination-based RTBH**: customer announces the victim /32 via iBGP with community `65535:666` (well-known `BLACKHOLE`, RFC 7999). Every edge router installs the route with next-hop `discard`/`null0`. The victim is effectively offline but the rest of the network survives.
- **Source-based RTBH** (uRPF loose-mode + RTBH): similar but blackholes by source IP, using uRPF loose-mode to drop traffic from a malicious source. Requires uRPF in the data path which is not always feasible.
- **BGP Flowspec** (RFC 5575/8955): announce a 5-tuple filter (src/dst IP, port, protocol, length) via BGP that gets compiled into ACLs at every edge. Far more surgical than RTBH but requires hardware support. Most modern Junos and IOS-XR can do this; many merchant-silicon routers cannot.
