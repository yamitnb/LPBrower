#!/usr/bin/env python3
"""
Compile official open-source Adblock filter lists (ABPVN, uBlock Origin uAssets,
Peter Lowe, EasyList, EasyPrivacy) into validated Chromium MV3 DeclarativeNetRequest
static rulesets + scoped cosmetic CSS under `src-tauri/assets/adblock/`.

Run manually whenever updating bundled filter lists:
    python scripts/build-adblock-assets.py
"""
from __future__ import annotations

import gzip
import json
import re
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "src-tauri" / "assets" / "adblock"
CACHE_DIR = Path(
    r"C:\Users\Admin\.gemini\antigravity\brain\6659144a-f4cf-41d2-a473-a1d186c1fdb7\scratch\filter_lists"
)

SOURCES = {
    "pgl": "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=adblockplus&showintro=0&mimetype=plaintext",
    "abpvn": "https://raw.githubusercontent.com/abpvn/abpvn/master/filter/abpvn.txt",
    "abpvn_ub": "https://raw.githubusercontent.com/abpvn/abpvn/master/filter/abpvn_ublock.txt",
    "easylist": "https://easylist.to/easylist/easylist.txt",
    "easyprivacy": "https://easylist.to/easylist/easyprivacy.txt",
    "ublock_filters": "https://ublockorigin.github.io/uAssets/filters/filters.txt",
    "ublock_privacy": "https://ublockorigin.github.io/uAssets/filters/privacy.txt",
    "ublock_badware": "https://ublockorigin.github.io/uAssets/filters/badware.txt",
    "ublock_quickfixes": "https://ublockorigin.github.io/uAssets/filters/quick-fixes.txt",
}

# Marketer & Seller portals that must NEVER have ads/pixels/scripts blocked.
EXCLUDED_MARKETER_DOMAINS = [
    "ads.google.com",
    "adwords.google.com",
    "analytics.google.com",
    "tagmanager.google.com",
    "adsmanager.facebook.com",
    "business.facebook.com",
    "ads.tiktok.com",
    "business.tiktok.com",
    "seller.shopee.vn",
    "seller.shopee.com",
    "banhang.shopee.vn",
    "sellercentral.amazon.com",
    "advertising.amazon.com",
]

# Critical auth, CAPTCHA, payment, and login SDK URLs that must NEVER be blocked.
CRITICAL_ALLOW_URL_FILTERS = [
    "||recaptcha.net^",
    "||www.gstatic.com^",
    "||hcaptcha.com^",
    "||challenges.cloudflare.com^",
    "||accounts.google.com^",
    "||apis.google.com^",
    "||js.stripe.com^",
    "||checkout.stripe.com^",
    "||m.stripe.com^",
    "||paypal.com^",
    "||paypalobjects.com^",
    "||appleid.cdn-apple.com^",
    "||login.microsoftonline.com^",
    "||auth0.com^",
    "||static.xx.fbcdn.net^",
    "||graph.facebook.com^",
    "||connect.facebook.net/*/sdk.js",
]

# Top Vietnamese news & entertainment sites for scoped ABPVN cosmetic CSS.
VN_COSMETIC_DOMAINS = {
    "24h.com.vn",
    "kenh14.vn",
    "vnexpress.net",
    "dantri.com.vn",
    "cafef.vn",
    "cafebiz.vn",
    "soha.vn",
    "genk.vn",
    "gamek.vn",
    "thanhnien.vn",
    "tuoitre.vn",
    "znews.vn",
    "zingnews.vn",
    "vietnamnet.vn",
    "eva.vn",
    "afamily.vn",
    "vtcnews.vn",
    "laodong.vn",
    "tienphong.vn",
    "baomoi.com",
    "tinhte.vn",
    "voz.vn",
    "nguoiduatin.vn",
    "saostar.vn",
    "2sao.vn",
    "vov.vn",
    "nld.com.vn",
    "sggp.org.vn",
    "plo.vn",
    "vneconomy.vn",
    "baogiaothong.vn",
    "bongdaplus.vn",
    "thethao247.vn",
    "nettruyen",
    "truyenfull",
}

# Adblock option -> Chrome DNR resourceType
RESOURCE_MAP = {
    "script": "script",
    "image": "image",
    "stylesheet": "stylesheet",
    "css": "stylesheet",
    "xmlhttprequest": "xmlhttprequest",
    "xhr": "xmlhttprequest",
    "subdocument": "sub_frame",
    "frame": "sub_frame",
    "media": "media",
    "font": "font",
    "websocket": "websocket",
    "ping": "ping",
    "beacon": "ping",
    "object": "object",
    "other": "other",
}

ALL_NON_MAIN_RESOURCES = [
    "script",
    "image",
    "stylesheet",
    "sub_frame",
    "xmlhttprequest",
    "ping",
    "media",
    "font",
    "object",
    "websocket",
    "other",
]

UNSUPPORTED_OPTIONS = {
    "redirect",
    "redirect-rule",
    "csp",
    "removeparam",
    "header",
    "replace",
    "badfilter",
    "elemhide",
    "ehide",
    "generichide",
    "ghide",
    "genericblock",
    "specifichide",
    "shide",
    "inline-script",
    "inline-font",
    "webrtc",
    "cname",
    "denyallow",
    "to",
    "permissions",
    "uritransform",
    "method",
    "ipaddress",
    "reason",
    "strict1p",
    "strict3p",
    "doc",
    "document",
    "all",
    "empty",
    "mp4",
    "noop",
}

VALID_DOMAIN_RE = re.compile(r"^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)+$")


def read_list(name: str) -> list[str]:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    p = CACHE_DIR / f"{name}.txt"
    if not p.exists():
        req = urllib.request.Request(SOURCES[name], headers={"User-Agent": "LPBrowser/2.1"})
        with urllib.request.urlopen(req, timeout=25) as r:
            p.write_bytes(r.read())
    return p.read_text(encoding="utf-8", errors="ignore").splitlines()


def is_ascii(s: str) -> bool:
    try:
        s.encode("ascii")
        return True
    except UnicodeEncodeError:
        return False


def valid_domain(d: str) -> bool:
    return bool(d) and len(d) <= 253 and is_ascii(d) and bool(VALID_DOMAIN_RE.match(d))


def parse_network_rule(raw_line: str) -> dict | None:
    line = raw_line.strip()
    if not line or line.startswith(("!", "#", "[")):
        return None
    # Cosmetic / scriptlet / HTML filters
    if any(sep in line for sep in ("##", "#@#", "#?#", "#$#", "#%#", "$$", "$@$")):
        return None

    is_allow = False
    if line.startswith("@@"):
        is_allow = True
        line = line[2:]

    # Split options at last unescaped '$'
    pattern = line
    opts_raw = ""
    if "$" in line:
        idx = line.rfind("$")
        pattern = line[:idx]
        opts_raw = line[idx + 1 :]

    # Skip regex patterns `/.../`
    if len(pattern) >= 2 and pattern.startswith("/") and pattern.endswith("/"):
        return None

    # Clean pattern
    if not pattern or len(pattern) < 4 or len(pattern) > 220 or not is_ascii(pattern):
        return None
    # Normalize `||*.example.com` -> `||example.com` (in DNR `||` already matches subdomains,
    # and `||*` is rejected by Chromium's url_pattern_index validator).
    if pattern.startswith("||*."):
        pattern = "||" + pattern[4:]
    # Chrome DNR doesn't support `||` without following host char or bare wildcards
    if pattern in ("*", "||", "||*", "||^", "|http://", "|https://"):
        return None
    # Strip trailing wildcard (no-op in DNR)
    while pattern.endswith("*") and len(pattern) > 4:
        pattern = pattern[:-1]
    if pattern.startswith("*") and len(pattern) <= 5:
        return None
    # Validate anchor syntax per Chromium IndexedRule::ParseUrlFilter:
    # `|` is only valid at start (`|` or `||`) or end (`|`); `||` must be followed by a host char.
    body = pattern
    if body.startswith("||"):
        body = body[2:]
        if not body or not body[0].isalnum():
            return None
    elif body.startswith("|"):
        body = body[1:]
    if body.endswith("|"):
        body = body[:-1]
    if not body or "|" in body:
        return None
    # Disallow whitespace or control characters in urlFilter
    if any(ord(c) <= 32 or ord(c) >= 127 for c in pattern):
        return None

    domain_type = None
    inc_domains: list[str] = []
    exc_domains: list[str] = []
    inc_types: set[str] = set()
    exc_types: set[str] = set()
    is_popup = False
    case_sensitive = False

    if opts_raw:
        for opt in opts_raw.split(","):
            opt = opt.strip()
            if not opt:
                continue
            neg = opt.startswith("~")
            body = opt[1:] if neg else opt
            key, eq, val = body.partition("=")
            key_l = key.lower()

            if key_l in ("important",):
                continue
            if key_l == "match-case":
                case_sensitive = not neg
                continue
            if key_l in ("third-party", "3p"):
                domain_type = "firstParty" if neg else "thirdParty"
                continue
            if key_l in ("first-party", "1p"):
                domain_type = "thirdParty" if neg else "firstParty"
                continue
            if key_l == "popup":
                if neg:
                    continue
                is_popup = True
                continue
            if key_l in ("domain", "from") and eq:
                for d in val.split("|"):
                    d = d.strip().lower()
                    if not d:
                        continue
                    d_neg = d.startswith("~")
                    d_host = d[1:] if d_neg else d
                    if d_host.endswith(".*") or "*" in d_host or "/" in d_host:
                        # Cannot express wildcard TLD in DNR initiatorDomains cleanly; skip whole rule if positive
                        if not d_neg:
                            return None
                        continue
                    if not valid_domain(d_host):
                        if not d_neg:
                            return None
                        continue
                    if d_neg:
                        exc_domains.append(d_host)
                    else:
                        inc_domains.append(d_host)
                continue
            if key_l in RESOURCE_MAP:
                rt = RESOURCE_MAP[key_l]
                if neg:
                    exc_types.add(rt)
                else:
                    inc_types.add(rt)
                continue
            if key_l in UNSUPPORTED_OPTIONS:
                return None
            # Unknown modifier: skip rule conservatively
            return None

    # Build resourceTypes cleanly
    if is_popup and not inc_types:
        # Pop-under / popup ad windows
        rtypes = ["main_frame", "sub_frame"]
        if domain_type is None:
            domain_type = "thirdParty"
    elif inc_types:
        rtypes = sorted(inc_types - exc_types)
    elif exc_types:
        rtypes = [r for r in ALL_NON_MAIN_RESOURCES if r not in exc_types]
    else:
        rtypes = None  # We will set excludedResourceTypes: ["main_frame"] on block rules

    if rtypes is not None and not rtypes:
        return None

    # Never allow a generic non-domain-scoped allow rule to blow open everything
    if is_allow and not pattern.startswith("||") and not inc_domains and len(pattern) < 10:
        return None

    # Safety: don't allow a bare short non-anchored substring without domain/3p scoping
    if not is_allow and not pattern.startswith("||") and not pattern.startswith("|http"):
        if len(pattern) < 8 and not inc_domains and domain_type != "thirdParty":
            return None

    condition: dict = {"urlFilter": pattern}
    if case_sensitive:
        condition["isUrlFilterCaseSensitive"] = True
    if domain_type:
        condition["domainType"] = domain_type
    if inc_domains:
        condition["initiatorDomains"] = sorted(set(inc_domains))
    if exc_domains:
        condition["excludedInitiatorDomains"] = sorted(set(exc_domains))

    if rtypes is not None:
        condition["resourceTypes"] = rtypes
    elif not is_allow:
        # Block all subresources, never main_frame on generic rules
        condition["excludedResourceTypes"] = ["main_frame"]

    return {
        "priority": 10 if is_allow else (2 if (inc_domains or is_popup) else 1),
        "action": {"type": "allow" if is_allow else "block"},
        "condition": condition,
    }


def rule_signature(rule: dict) -> str:
    return json.dumps(
        [rule["action"]["type"], rule["condition"]],
        separators=(",", ":"),
        sort_keys=True,
    )


def extract_vn_cosmetic_css(lines: list[str]) -> str:
    """Extract safe, pure-CSS hiding selectors from ABPVN for Vietnamese sites."""
    selectors: list[str] = []
    seen: set[str] = set()
    # Skip procedural/extended CSS syntax unsupported by native Chromium stylesheets
    bad_tokens = (
        ":has-text(",
        ":matches-css(",
        ":upward(",
        ":xpath(",
        ":style(",
        ":remove(",
        ":watch-attr(",
        ":min-text-length(",
        ":-abp-",
        "+js(",
        ":matches-path(",
        ":others(",
    )
    for raw in lines:
        line = raw.strip()
        if not line or line.startswith(("!", "[")):
            continue
        if "#@#" in line or "#?#" in line or "#$#" in line or "#%#" in line:
            continue
        if "##" not in line:
            continue
        dom_part, sel = line.split("##", 1)
        sel = sel.strip()
        if not sel or len(sel) > 220 or any(tok in sel for tok in bad_tokens):
            continue
        if "{" in sel or "}" in sel or "/*" in sel or "\n" in sel:
            continue
        # Only include selectors that apply globally in ABPVN or to our VN_COSMETIC_DOMAINS
        if dom_part.strip():
            doms = [d.strip().lower() for d in dom_part.split(",") if d.strip() and not d.strip().startswith("~")]
            if not any(any(v in d for v in VN_COSMETIC_DOMAINS) for d in doms):
                continue
        # Skip overly broad bare element/body selectors
        if sel.lower() in ("body", "html", "div", "iframe", "img", "a", "section", "aside", "header", "footer"):
            continue
        if sel not in seen:
            seen.add(sel)
            selectors.append(sel)

    # Extra high-impact Vietnamese ad banner selectors (24h, Kenh14, VnExpress, CafeF, Dantri, Phim/Truyen)
    curated_vn = [
        'iframe[src*="html5_banner"]',
        'iframe[src*="/upload/"][src*="banner"]',
        'iframe[src*="admicro"]',
        'iframe[src*="amcdn.vn"]',
        'iframe[src*="eclick.vn"]',
        'iframe[src*="adtima.vn"]',
        'iframe[src*="ants.vn"]',
        'iframe[src*="contineljs"]',
        'iframe[src*="deqik.com"]',
        'iframe[src*="populartooth"]',
        'iframe[src*="cuddlybasis"]',
        'iframe[src*="baotinmanhhai"]',
        '[id^="admzone"]',
        '[id^="admicro"]',
        '[class*="admicro"]',
        '[id^="eclick"]',
        '[class*="eclick"]',
        '[id^="adtima"]',
        '[class*="adtima"]',
        '[id^="ads_zone"]',
        '[id^="zone-"]',
        '.adm-banner',
        '.adm-sponsor',
        '.k14-ad-box',
        '.banner-ads',
        '.ads-banner',
        '.box-ads',
        '.adv-box',
        '.qc-banner',
        '.float-ads',
        '.balloon-ads',
        '.catfish-banner',
        '#catfish',
        '#catfish-banner',
        '.banner_24h',
        '.banner-24h',
        '[id^="adv_"]',
        '[id^="ads_24h"]',
        '[class^="adv-"]',
        '.vne-ads',
        '.box_quangcao',
        '.quang-cao',
        '.quangcao',
    ]
    for s in curated_vn:
        if s not in seen:
            seen.add(s)
            selectors.append(s)

    # Emit in chunks of 25 selectors per CSS block so one bad selector never invalidates the whole sheet
    blocks = ["/* LPBrowser VN Cosmetic Rules (ABPVN + curated) */"]
    for i in range(0, len(selectors), 20):
        chunk = ",\n".join(selectors[i : i + 20])
        blocks.append(f"{chunk} {{\n  display: none !important;\n}}")
    return "\n\n".join(blocks) + "\n"


def build_cosmetic_global(easylist_lines: list[str], ublock_lines: list[str], abpvn_lines: list[str]) -> str:
    """Build global cosmetic CSS combining curated selectors + EasyList/uBlock/ABPVN generic hiding rules."""
    curated = [
        # Google Ads / AdSense / GPT / DoubleClick
        'ins.adsbygoogle',
        'iframe[id^="google_ads_iframe"]',
        'div[id^="google_ads_iframe"]',
        'div[id^="div-gpt-ad"]',
        '[data-google-query-id]',
        'iframe[src*="doubleclick.net"]',
        'iframe[src*="googlesyndication.com"]',
        'iframe[src*="googleadservices.com"]',
        'iframe[src*="taboola.com"]',
        'iframe[src*="outbrain.com"]',
        'iframe[src*="mgid.com"]',
        'iframe[src*="criteo."]',
        'iframe[src*="adnxs.com"]',
        'iframe[src*="rubiconproject.com"]',
        'iframe[src*="pubmatic.com"]',
        'iframe[src*="openx.net"]',
        'iframe[src*="amazon-adsystem.com"]',
        'iframe[src*="adsterra"]',
        'iframe[src*="propellerads"]',
        'iframe[src*="exoclick"]',
        'iframe[src*="popads"]',
        'iframe[src*="admicro"]',
        'iframe[src*="amcdn.vn"]',
        'iframe[src*="eclick.vn"]',
        'iframe[src*="adtima.vn"]',
        'iframe[src*="html5_banner"]',
        'a[href*="doubleclick.net/pcs/click"]',
        'a[href*="googleadservices.com/pagead/aclk"]',
        'a[href*="eclick.vn/click"]',
        'a[href*="admicro.vn/click"]',
        '.trc_rbox_container',
        '.OUTBRAIN',
        '[id^="taboola-"]',
        '[id^="mgid-"]',
        '[id^="rc-widget-"]',
        '[id^="admzone"]',
        '.adm-banner',
        '.eclick-banner',
        '.GoogleActiveViewElement',
        '.adsbygoogle-noablate',
        # 2fa.live & Vietnamese MMO/utility direct banner containers
        '.sidebar-ads',
        '.ads_vn',
        '.ads_th',
        'img[alt="Advertisement"]',
        'img[alt="Vietnam Advertisement"]',
        'img[alt="Thai Advertisement"]',
        'img[src*="pub-8f4153723cd0473bb68fb9b01dee732d.r2.dev"]',
        # Reddit promoted ads
        'shreddit-ad-post',
        'shreddit-comments-page-ad',
        'shreddit-async-loader[bundlename="feed_ad"]',
        '.promotedlink',
        # Streaming / manga catfish & overlay banners
        '.catfish-ad',
        '.catfish-banner',
        '#catfish-banner',
        '.floating-ad-banner',
        '.sticky-bottom-ad',
        '.popup-banner-ads',
        'img[src*="_ads_banner"]',
        'img[src*="_ad_banner"]',
        'embed[src*="_ads_banner"]',
        'object[data*="_ads_banner"]',
    ]

    bad_tokens = (
        ":has-text(",
        ":matches-css(",
        ":upward(",
        ":xpath(",
        ":style(",
        ":remove(",
        ":watch-attr(",
        ":min-text-length(",
        ":-abp-",
        "+js(",
        ":matches-path(",
        ":others(",
        ":not(",
        "/*",
        "{",
        "}",
        "\n",
    )
    # Never hide ultra-generic single-word classes/ids that sites use for anti-adblock checks or layout
    skip_selectors = {
        ".ad", ".ads", "#ad", "#ads", ".adv", "#adv", ".advert", ".banner", "#banner",
        ".sponsor", ".sponsored", ".promo", ".popup", ".modal", ".overlay",
        "body", "html", "div", "iframe", "img", "a", "section", "aside", "header", "footer",
    }

    selectors: list[str] = []
    seen: set[str] = set()
    for s in curated:
        if s not in seen:
            seen.add(s)
            selectors.append(s)

    for raw in abpvn_lines + ublock_lines + easylist_lines:
        line = raw.strip()
        if not line.startswith("##"):
            continue
        sel = line[2:].strip()
        if not sel or len(sel) < 4 or len(sel) > 160:
            continue
        if sel.lower() in skip_selectors:
            continue
        if any(tok in sel for tok in bad_tokens):
            continue
        if not is_ascii(sel):
            continue
        # Only accept clean class/id/attribute/element selectors (no stray spaces that might break grouping)
        if not re.match(r'^[#.\[a-zA-Z][a-zA-Z0-9_\-.,#\[\]="\'*^$|~>:() ]+$', sel):
            continue
        # Balanced quotes and brackets check
        if sel.count("[") != sel.count("]") or sel.count("(") != sel.count(")"):
            continue
        if sel.count('"') % 2 != 0 or sel.count("'") % 2 != 0:
            continue
        if sel not in seen:
            seen.add(sel)
            selectors.append(sel)

    # Chunk into groups of 40 selectors per CSS rule so Blink parses fast and stays resilient
    blocks = [f"/* LPBrowser Global Cosmetic Rules ({len(selectors)} selectors: curated + EasyList + uBlock + ABPVN) */"]
    for i in range(0, len(selectors), 40):
        chunk = ",\n".join(selectors[i : i + 40])
        blocks.append(f"{chunk} {{\n  display: none !important;\n}}")
    return "\n\n".join(blocks) + "\n"


def build_cosmetic_search() -> str:
    return """/* LPBrowser Search Sponsored Ads Cosmetic Rules */
#tads,
#tadsb,
#tvcap,
#bottomads,
[data-text-ad],
.commercial-unit-desktop-top,
.commercial-unit-desktop-rhs,
.commercial-unit-mobile-top,
.cu-container,
.pla-unit-container,
.mnr-c.pla-unit,
.b_ad,
.b_adTop,
.b_adBottom,
.b_adSlug,
li.b_ad,
.ad_sc,
#ads,
li[data-layout="ad"],
.results--ads {
  display: none !important;
}
"""


def build_cosmetic_yt() -> str:
    return """/* LPBrowser YouTube Cosmetic Ad Rules */
ytd-ad-slot-renderer,
ytd-in-feed-ad-layout-renderer,
ytd-banner-promo-renderer,
ytd-statement-banner-renderer,
ytd-promoted-sparkles-web-renderer,
ytd-promoted-sparkles-text-search-renderer,
ytd-promoted-video-renderer,
ytd-display-ad-renderer,
ytd-compact-promoted-video-renderer,
ytd-companion-slot-renderer,
ytd-action-companion-ad-renderer,
ytd-video-masthead-ad-v3-renderer,
ytd-primetime-promo-renderer,
ytd-merch-shelf-renderer,
ytd-rich-item-renderer:has(ytd-ad-slot-renderer),
ytd-rich-section-renderer:has(ytd-statement-banner-renderer),
ytd-rich-section-renderer:has(ytd-brand-video-shelf-renderer),
ytd-rich-section-renderer:has(ytd-ad-slot-renderer),
ytd-reel-video-renderer:has(ytd-ad-slot-renderer),
#masthead-ad,
#player-ads,
#panels ytd-engagement-panel-section-list-renderer[target-id="engagement-panel-ads"],
.ytd-ad-slot-renderer,
.ytp-ad-overlay-container,
.ytp-ad-text-overlay,
.ytp-ad-image-overlay,
.ytp-ad-action-interstitial,
.ytp-ad-progress-list,
.ytp-suggested-action-badge,
.video-ads.ytp-ad-module > .ytp-ad-overlay-container,
tp-yt-paper-dialog:has(ytd-enforcement-message-view-model),
ytd-enforcement-message-view-model {
  display: none !important;
}
"""


def build_yt_adblock_js() -> str:
    return r"""/* LPBrowser Built-in YouTube Ad Blocker (MAIN world, youtube.com only) */
(function () {
  "use strict";
  if (window.__lpbYtAdblockInstalled) return;
  window.__lpbYtAdblockInstalled = true;

  const AD_KEYS = [
    "adPlacements",
    "playerAds",
    "adSlots",
    "adBreakHeartbeatParams",
    "adBreakParams",
  ];

  const AD_RENDERER_KEYS = new Set([
    "adSlotRenderer",
    "promotedSparklesWebRenderer",
    "promotedSparklesTextSearchRenderer",
    "inFeedAdLayoutRenderer",
    "bannerPromoRenderer",
    "statementBannerRenderer",
    "displayAdRenderer",
    "compactPromotedVideoRenderer",
    "videoMastheadAdV3Renderer",
    "actionCompanionAdRenderer",
    "linearAdSequenceRenderer",
    "instreamVideoAdRenderer",
  ]);

  function cleanObject(obj, depth) {
    if (!obj || typeof obj !== "object" || depth > 14) return obj;
    if (Array.isArray(obj)) {
      for (let i = obj.length - 1; i >= 0; i--) {
        const item = obj[i];
        if (item && typeof item === "object") {
          let isAd = false;
          for (const k of Object.keys(item)) {
            if (AD_RENDERER_KEYS.has(k)) {
              isAd = true;
              break;
            }
          }
          if (isAd) {
            obj.splice(i, 1);
            continue;
          }
          cleanObject(item, depth + 1);
        }
      }
      return obj;
    }
    for (const k of AD_KEYS) {
      if (k in obj) {
        if (Array.isArray(obj[k])) {
          obj[k] = [];
        } else {
          delete obj[k];
        }
      }
    }
    for (const k of Object.keys(obj)) {
      if (AD_RENDERER_KEYS.has(k)) {
        delete obj[k];
      } else {
        const v = obj[k];
        if (v && typeof v === "object") {
          cleanObject(v, depth + 1);
        }
      }
    }
    return obj;
  }

  // 1. Trap initial ytInitialPlayerResponse & ytInitialData before YT consumes them
  function trapGlobal(name) {
    let current = window[name];
    if (current && typeof current === "object") cleanObject(current, 0);
    try {
      Object.defineProperty(window, name, {
        configurable: true,
        enumerable: true,
        get() {
          return current;
        },
        set(val) {
          if (val && typeof val === "object") {
            try {
              cleanObject(val, 0);
            } catch {}
          }
          current = val;
        },
      });
    } catch {}
  }
  trapGlobal("ytInitialPlayerResponse");
  trapGlobal("ytInitialData");

  // Also clean ytplayer.config.args.raw_player_response if YouTube writes it
  let ytPlayerVal = window.ytplayer;
  function scrubYtPlayer(yp) {
    try {
      const args = yp && yp.config && yp.config.args;
      if (!args) return;
      if (args.raw_player_response && typeof args.raw_player_response === "object") {
        cleanObject(args.raw_player_response, 0);
      }
      if (typeof args.player_response === "string" && (args.player_response.includes("adPlacements") || args.player_response.includes("playerAds"))) {
        const parsed = JSON.parse(args.player_response);
        cleanObject(parsed, 0);
        args.player_response = JSON.stringify(parsed);
      }
    } catch {}
  }
  if (ytPlayerVal) scrubYtPlayer(ytPlayerVal);
  try {
    Object.defineProperty(window, "ytplayer", {
      configurable: true,
      enumerable: true,
      get() { return ytPlayerVal; },
      set(val) {
        ytPlayerVal = val;
        scrubYtPlayer(val);
      },
    });
  } catch {}

  const AD_TEXT_MARKERS = [
    '"adPlacements"',
    '"playerAds"',
    '"adSlots"',
    '"adBreakHeartbeatParams"',
    '"adSlotRenderer"',
    '"promotedSparkles',
  ];
  function hasAdMarkers(txt) {
    for (let i = 0; i < AD_TEXT_MARKERS.length; i++) {
      if (txt.includes(AD_TEXT_MARKERS[i])) return true;
    }
    return false;
  }

  // 2. Patch fetch() for YouTube player/browse/next/search/reel/ad_break API responses
  const origFetch = window.fetch;
  if (typeof origFetch === "function") {
    window.fetch = async function (input, init) {
      const url = typeof input === "string" ? input : (input && input.url) || "";
      if (url.includes("/youtubei/v1/ad_break")) {
        return new Response("{}", {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }
      const isYtApi =
        url.includes("/youtubei/v1/player") ||
        url.includes("/youtubei/v1/next") ||
        url.includes("/youtubei/v1/browse") ||
        url.includes("/youtubei/v1/search") ||
        url.includes("/youtubei/v1/reel/reel_item_watch");
      const resp = await origFetch.apply(this, arguments);
      if (!isYtApi) return resp;
      try {
        const ct = resp.headers.get("content-type") || "";
        if (!ct.includes("json")) return resp;
        const text = await resp.clone().text();
        if (!hasAdMarkers(text)) {
          return resp;
        }
        const data = JSON.parse(text);
        cleanObject(data, 0);
        const body = JSON.stringify(data);
        const headers = new Headers(resp.headers);
        headers.delete("content-length");
        return new Response(body, {
          status: resp.status,
          statusText: resp.statusText,
          headers,
        });
      } catch {
        return resp;
      }
    };
  }

  // 3. Patch Response.prototype.json, Response.prototype.text, XMLHttpRequest, and JSON.parse
  const origJson = Response.prototype.json;
  Response.prototype.json = function () {
    return origJson.apply(this, arguments).then((data) => {
      try {
        if (
          data &&
          typeof data === "object" &&
          ("adPlacements" in data || "playerAds" in data || "adSlots" in data || "adBreakHeartbeatParams" in data)
        ) {
          cleanObject(data, 0);
        }
      } catch {}
      return data;
    });
  };

  const origText = Response.prototype.text;
  Response.prototype.text = function () {
    return origText.apply(this, arguments).then((txt) => {
      try {
        if (typeof txt === "string" && txt.startsWith("{") && hasAdMarkers(txt)) {
          const parsed = origJsonParse(txt);
          cleanObject(parsed, 0);
          return JSON.stringify(parsed);
        }
      } catch {}
      return txt;
    });
  };

  const origJsonParse = JSON.parse;
  JSON.parse = function (text, reviver) {
    const res = origJsonParse.apply(this, arguments);
    try {
      if (typeof text === "string" && res && typeof res === "object" && hasAdMarkers(text)) {
        cleanObject(res, 0);
      }
    } catch {}
    return res;
  };

  const origXhrOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url) {
    this.__lpbYtUrl = typeof url === "string" ? url : String(url || "");
    return origXhrOpen.apply(this, arguments);
  };
  const origRespText = Object.getOwnPropertyDescriptor(XMLHttpRequest.prototype, "responseText");
  if (origRespText && origRespText.get) {
    Object.defineProperty(XMLHttpRequest.prototype, "responseText", {
      configurable: true,
      get() {
        const txt = origRespText.get.call(this);
        const u = this.__lpbYtUrl || "";
        if (
          txt &&
          (u.includes("/youtubei/v1/player") ||
            u.includes("/youtubei/v1/next") ||
            u.includes("/youtubei/v1/browse") ||
            u.includes("/youtubei/v1/reel/reel_item_watch") ||
            u.includes("/youtubei/v1/ad_break")) &&
          hasAdMarkers(txt)
        ) {
          try {
            const parsed = origJsonParse(txt);
            cleanObject(parsed, 0);
            return JSON.stringify(parsed);
          } catch {}
        }
        return txt;
      },
    });
  }

  // 4. Real-time safety-net skipper for server-stitched ads + enforcement popup killer
  let prevMuted = null;
  function sweepAdPlayer() {
    scrubYtPlayer(window.ytplayer);
    const player = document.querySelector(".html5-video-player");
    const isAd =
      player &&
      (player.classList.contains("ad-showing") ||
        player.classList.contains("ad-interrupting") ||
        !!player.querySelector(".ytp-ad-player-overlay, .ytp-ad-player-overlay-layout"));
    if (isAd) {
      const video = player.querySelector("video.html5-main-video");
      if (video) {
        if (prevMuted === null) prevMuted = video.muted;
        video.muted = true;
        try {
          video.playbackRate = 16;
        } catch {}
        if (Number.isFinite(video.duration) && video.duration > 0) {
          try {
            video.currentTime = video.duration;
          } catch {}
        }
      }
      const skipBtns = document.querySelectorAll(
        ".ytp-skip-ad-button, .ytp-ad-skip-button, .ytp-ad-skip-button-modern, button[id^='skip-button'], .ytp-ad-overlay-close-button"
      );
      skipBtns.forEach((btn) => {
        try {
          btn.click();
        } catch {}
      });
    } else if (prevMuted !== null) {
      const video = document.querySelector("video.html5-main-video");
      if (video) {
        video.muted = prevMuted;
        if (video.playbackRate > 4) video.playbackRate = 1;
      }
      prevMuted = null;
    }

    // Dismiss anti-adblock enforcement popup if present
    const dialog = document.querySelector("tp-yt-paper-dialog ytd-enforcement-message-view-model");
    if (dialog) {
      const wrapper = dialog.closest("tp-yt-paper-dialog");
      if (wrapper) wrapper.remove();
      const backdrop = document.querySelector("tp-yt-iron-overlay-backdrop");
      if (backdrop) backdrop.remove();
      const video = document.querySelector("video.html5-main-video");
      if (video && video.paused) {
        try {
          video.play();
        } catch {}
      }
    }
  }

  function startObserver() {
    sweepAdPlayer();
    const obs = new MutationObserver(sweepAdPlayer);
    if (document.documentElement) {
      obs.observe(document.documentElement, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["class"],
      });
    }
    window.addEventListener("yt-navigate-finish", sweepAdPlayer);
    setInterval(sweepAdPlayer, 250);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", startObserver, { once: true });
  } else {
    startObserver();
  }
})();
"""


def build_curated_core_rules() -> list[dict]:
    """High-priority safety allowlists + essential rules that must always exist."""
    rules: list[dict] = []

    # 1. Priority 100: Allow everything initiated by Marketer / Ads Manager / Seller portals
    rules.append(
        {
            "priority": 100,
            "action": {"type": "allow"},
            "condition": {
                "initiatorDomains": EXCLUDED_MARKETER_DOMAINS,
            },
        }
    )
    rules.append(
        {
            "priority": 100,
            "action": {"type": "allowAllRequests"},
            "condition": {
                "requestDomains": EXCLUDED_MARKETER_DOMAINS,
                "resourceTypes": ["main_frame", "sub_frame"],
            },
        }
    )

    # 2. Priority 90: Never break critical Auth, CAPTCHA, Payment, or FB Login SDK
    for uf in CRITICAL_ALLOW_URL_FILTERS:
        rules.append(
            {
                "priority": 90,
                "action": {"type": "allow"},
                "condition": {"urlFilter": uf},
            }
        )

    # 3. Curated extra Vietnamese, 24h, Admicro rotating, and benchmark rules
    extra_blocks = [
        # 2fa.live & sister Vietnamese MMO tool sites direct R2 banner host
        ("||pub-8f4153723cd0473bb68fb9b01dee732d.r2.dev^", None, None),
        # 24h.com.vn first-party HTML5 banners
        ("||cdn.24h.com.vn/upload/*html5_banner", None, None),
        ("||cdn.24h.com.vn/*banner*", None, ["sub_frame", "script"]),
        ("||ads.24h.com.vn^", None, None),
        # Admicro / VCCorp rotating ad & anti-adblock domains observed live on Kenh14/CafeF/Soha/GenK
        ("||amcdn.vn^", None, None),
        ("||admicro.vn^", None, None),
        ("||contineljs.com^", None, None),
        ("||deqik.com^", None, None),
        ("||trunkpkg.com^", None, None),
        ("||sourcetobin.com^", None, None),
        ("||cuddlybasis.com^", None, None),
        ("||formalhood.com^", None, None),
        ("||populartooth.com^", None, None),
        ("||bubblyzucchini.com^", None, None),
        ("||farawayinstallation.com^", None, None),
        ("||lg.nanda.vn^", None, None),
        ("||eth.cnnd.vn^", None, None),
        ("||event.mediacdn.vn^", None, None),
        ("||cdnstoremedia.com^", "thirdParty", None),
        ("||adtrafficquality.google^", None, None),
        ("||eclick.vn^", None, None),
        ("||adtima.vn^", None, None),
        ("||ants.vn^", None, None),
        ("||antsomi.com^", None, None),
        ("||polyad.net^", None, None),
        ("||yomedia.vn^", None, None),
        ("||novanet.vn^", None, None),
        ("||urekamedia.com^", None, None),
        ("||ambient-platform.com^", None, None),
        ("||blueseed.tv^", None, None),
        ("||adnetwork.vn^", None, None),
        ("||adx.vn^", None, None),
        ("||qc.coccoc.com^", None, None),
        ("||adbro.me^", None, None),
        # Major global analytics, telemetry, error-trackers & ad tags
        ("||googletagmanager.com^", "thirdParty", None),
        ("||google-analytics.com^", None, None),
        ("||analytics.google.com^", "thirdParty", None),
        ("||static.cloudflareinsights.com^", None, None),
        ("||sentry-cdn.com^", "thirdParty", None),
        ("||sentry.io^", "thirdParty", None),
        ("||bugsnag.com^", "thirdParty", None),
        ("||d2wy8f7a9ursnm.cloudfront.net^", "thirdParty", None),
        ("||connect.facebook.net/*/fbevents.js", "thirdParty", ["script"]),
        ("||facebook.com/tr", "thirdParty", ["image", "ping", "xmlhttprequest", "script", "other"]),
        # Yandex & Baidu ad networks
        ("||an.yandex.ru^", None, None),
        ("||mc.yandex.ru^", None, None),
        ("||yandexadexchange.net^", None, None),
        ("||yastatic.net/partner-code-bundles/", None, None),
        ("||yastatic.net/safeframe-bundles/", None, None),
        ("||pos.baidu.com^", None, None),
        ("||cpro.baidu.com^", None, None),
        ("||hm.baidu.com^", None, None),
        # Universal high-signal banner / ad-server path patterns (subresources only)
        ("_ads_banner", None, None),
        ("_ad_banner", None, None),
        ("-ads-banner", None, None),
        ("-ad-banner", None, None),
        ("/ads_banner", None, None),
        ("/ad_banner", None, None),
        ("/adserver/", None, None),
        ("/adservers/", None, None),
        ("/pagead/js/", None, None),
        ("/pagead/ads", None, None),
        ("/html5_banner", None, None),
        ("/popunder.", None, ["script", "sub_frame"]),
        # YouTube & Google pagead ad endpoints
        ("||youtube.com/youtubei/v1/ad_break", None, ["xmlhttprequest", "other"]),
        ("||youtube.com/pagead/", None, ["xmlhttprequest", "script", "image", "ping", "other", "media"]),
        ("||youtube.com/api/stats/ads", None, ["xmlhttprequest", "script", "image", "ping", "other", "media"]),
        ("||youtube.com/ptracking", None, ["xmlhttprequest", "script", "image", "ping", "other", "media"]),
        ("||youtube.com/get_midroll_info", None, ["xmlhttprequest", "script", "image", "ping", "other", "media"]),
        ("||google.com/pagead/", None, ["xmlhttprequest", "script", "image", "ping", "other", "sub_frame"]),
        ("||google.com.vn/pagead/", None, ["xmlhttprequest", "script", "image", "ping", "other", "sub_frame"]),
    ]
    for uf, dtype, rtypes in extra_blocks:
        cond: dict = {"urlFilter": uf}
        if dtype:
            cond["domainType"] = dtype
        if rtypes:
            cond["resourceTypes"] = rtypes
        else:
            cond["excludedResourceTypes"] = ["main_frame"]
        rules.append({"priority": 3, "action": {"type": "block"}, "condition": cond})

    return rules


def assign_ids(rules: list[dict]) -> list[dict]:
    out = []
    for idx, r in enumerate(rules, start=1):
        out.append({"id": idx, **r})
    return out


def collect_rules(lines: list[str], seen: set[str], limit: int) -> list[dict]:
    out: list[dict] = []
    for raw in lines:
        if len(out) >= limit:
            break
        r = parse_network_rule(raw)
        if not r:
            continue
        sig = rule_signature(r)
        if sig in seen:
            continue
        seen.add(sig)
        out.append(r)
    return out


def write_gz_json(path: Path, data: object) -> tuple[int, int]:
    raw = json.dumps(data, separators=(",", ":"), ensure_ascii=True).encode("ascii")
    comp = gzip.compress(raw, compresslevel=9, mtime=0)
    path.write_bytes(comp)
    return len(raw), len(comp)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    lists = {name: read_list(name) for name in SOURCES}

    seen: set[str] = set()

    # Ruleset 1: rules_core.json (Safety allowlists + curated + ABPVN + uBlock + Peter Lowe)
    core_rules = build_curated_core_rules()
    for r in core_rules:
        seen.add(rule_signature(r))

    core_rules.extend(collect_rules(lists["abpvn"] + lists["abpvn_ub"], seen, limit=2000))
    # Prioritize 100% of popup/pop-under rules across uBlock + EasyList so streaming/MMO sites never pop ad tabs
    all_popups = [
        l for l in (lists["ublock_badware"] + lists["ublock_filters"] + lists["easylist"])
        if "$popup" in l or ",popup" in l
    ]
    core_rules.extend(collect_rules(all_popups, seen, limit=2500))
    core_rules.extend(
        collect_rules(
            lists["ublock_quickfixes"]
            + lists["ublock_filters"]
            + lists["ublock_privacy"]
            + lists["ublock_badware"],
            seen,
            limit=7000,
        )
    )
    core_rules.extend(collect_rules(lists["pgl"], seen, limit=4000))

    # Ruleset 2: rules_easylist.json (EasyList network rules, capped to stay comfortably under 30,000 total)
    budget_remaining = max(0, 28500 - len(core_rules))
    easylist_budget = int(budget_remaining * 0.62)
    privacy_budget = budget_remaining - easylist_budget

    # Prioritize EasyList exceptions + generic ad-path rules first, then domain rules
    el_paths = [
        l for l in lists["easylist"]
        if l.startswith(("@@", "/", "_", "-", "&ad", "?ad", ".ad")) and "$" not in l.split("domain=")[0]
    ]
    el_domains = [l for l in lists["easylist"] if l.startswith("||")]
    el_rest = [l for l in lists["easylist"] if not l.startswith(("@@", "||", "/", "_", "-", "&ad", "?ad", ".ad"))]
    easylist_rules = collect_rules(el_paths + el_domains + el_rest, seen, limit=easylist_budget)

    # Ruleset 3: rules_privacy.json (EasyPrivacy network rules)
    ep_priority = [l for l in lists["easyprivacy"] if l.startswith(("@@", "||"))]
    ep_rest = [l for l in lists["easyprivacy"] if not l.startswith(("@@", "||"))]
    privacy_rules = collect_rules(ep_priority + ep_rest, seen, limit=privacy_budget)

    core_numbered = assign_ids(core_rules)
    easylist_numbered = assign_ids(easylist_rules)
    privacy_numbered = assign_ids(privacy_rules)

    total = len(core_numbered) + len(easylist_numbered) + len(privacy_numbered)
    assert total <= 29500, f"Total rules {total} exceeds safe Chrome MV3 30,000 guarantee!"

    r1_raw, r1_gz = write_gz_json(OUT_DIR / "rules_core.json.gz", core_numbered)
    r2_raw, r2_gz = write_gz_json(OUT_DIR / "rules_easylist.json.gz", easylist_numbered)
    r3_raw, r3_gz = write_gz_json(OUT_DIR / "rules_privacy.json.gz", privacy_numbered)

    global_css = build_cosmetic_global(lists["easylist"], lists["ublock_filters"], lists["abpvn"] + lists["abpvn_ub"])
    global_css_bytes = global_css.encode("utf-8")
    global_css_gz = gzip.compress(global_css_bytes, compresslevel=9, mtime=0)
    (OUT_DIR / "cosmetic_global.css.gz").write_bytes(global_css_gz)
    if (OUT_DIR / "cosmetic_global.css").exists():
        (OUT_DIR / "cosmetic_global.css").unlink()
    (OUT_DIR / "cosmetic_vn.css").write_text(
        extract_vn_cosmetic_css(lists["abpvn"] + lists["abpvn_ub"]), encoding="utf-8"
    )
    (OUT_DIR / "cosmetic_search.css").write_text(build_cosmetic_search(), encoding="utf-8")
    (OUT_DIR / "cosmetic_yt.css").write_text(build_cosmetic_yt(), encoding="utf-8")
    (OUT_DIR / "yt_adblock.js").write_text(build_yt_adblock_js(), encoding="utf-8")

    print(f"rules_core.json:     {len(core_numbered):5d} rules | raw={r1_raw:7d} B | gz={r1_gz:6d} B")
    print(f"rules_easylist.json: {len(easylist_numbered):5d} rules | raw={r2_raw:7d} B | gz={r2_gz:6d} B")
    print(f"rules_privacy.json:  {len(privacy_numbered):5d} rules | raw={r3_raw:7d} B | gz={r3_gz:6d} B")
    print(f"TOTAL DNR RULES:     {total:5d} rules | total gz={r1_gz + r2_gz + r3_gz:,} B")
    print(f"cosmetic_global.css: raw={len(global_css_bytes):7d} B | gz={len(global_css_gz):6d} B")
    for fn in ("cosmetic_vn.css", "cosmetic_search.css", "cosmetic_yt.css", "yt_adblock.js"):
        print(f"{fn:20s} {(OUT_DIR / fn).stat().st_size:7d} B")


if __name__ == "__main__":
    main()
