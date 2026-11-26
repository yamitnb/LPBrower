/* LPBrowser Built-in YouTube Ad Blocker (MAIN world, youtube.com only) */
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
