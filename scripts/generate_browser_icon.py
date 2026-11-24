import math
import os
import struct
from PIL import Image, ImageDraw

PROJECT_DIR = r"g:\Vibe Tool\LPBrower"
OUT_TAURI_PNG = os.path.join(PROJECT_DIR, r"src-tauri\icons\browser-icon.png")
OUT_TAURI_ICO = os.path.join(PROJECT_DIR, r"src-tauri\icons\browser-icon.ico")
OUT_PUBLIC = os.path.join(PROJECT_DIR, "public")
OUT_ARTIFACT = r"C:\Users\Admin\.gemini\antigravity\brain\6659144a-f4cf-41d2-a473-a1d186c1fdb7\chromium_lp_browser_icon.png"


def lerp(a, b, t):
    return a + (b - a) * max(0.0, min(1.0, t))


def lerp_rgb(c1, c2, t):
    t = max(0.0, min(1.0, t))
    return (
        int(c1[0] + (c2[0] - c1[0]) * t),
        int(c1[1] + (c2[1] - c1[1]) * t),
        int(c1[2] + (c2[2] - c1[2]) * t),
    )


def render_chromium_lp_icon(size=1024, draw_lp=True):
    """
    Renders a crisp, mathematically exact Chromium-core 3-blade aperture icon
    with LPBrower signature cobalt/indigo/cyan gradient palette and crisp LP core mark.
    """
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()

    cx = (size - 1) / 2.0
    cy = (size - 1) / 2.0
    scale = size / 2.0

    R_OUT = 0.945
    R_WHITE = 0.475
    R_CORE = 0.378
    AA = 1.6 / scale  # anti-alias band in normalized units

    # 3 Chromium tangent angles (in radians, y-down screen coords):
    # Tangent line k has outward normal n_k and tangent direction d_k:
    # For standard Chromium wheel:
    #   Seam 0: horizontal rightwards from top of inner circle (0, -R_WHITE) -> dir (1, 0), normal pointing up (0, -1)
    #   Seam 1: rotated +120 deg CW
    #   Seam 2: rotated +240 deg CW
    seams = []
    for k in range(3):
        ang = math.radians(k * 120.0)
        # Tangent point on inner white ring when k=0 is (0, -R_WHITE)
        # Rotate (0, -1) by ang CW: (sin(ang), -cos(ang))
        nx = math.sin(ang)
        ny = -math.cos(ang)
        # Tangent ray direction pointing away from tangent point: (cos(ang), sin(ang))
        dx = math.cos(ang)
        dy = math.sin(ang)
        seams.append((nx, ny, dx, dy))

    def point_blade(x, y):
        """
        Determine which of the 3 Chromium blades (0=top, 1=bottom-right, 2=bottom-left)
        contains point (x, y) outside R_WHITE, plus distance to the leading tangent seam
        for subtle fold shadow.
        """
        # Signed distance to each tangent line: dot((x,y), n_k) - R_WHITE
        # A point is on the "clockwise-past" side of seam k if (x*dx + y*dy) >= 0 and signed_n <= 0
        # Exact Chromium partition:
        # Blade 0 (Top): above seam 0 (n0 >= 0) AND behind/above seam 2 (n2 < 0 or ...)
        # Cleanest exact test using the 3 tangent half-planes:
        # Each tangent line k splits the plane into:
        #   s_k = x * nx_k + y * ny_k - R_WHITE   (positive = on the tangent-point side, negative = on the blade side)
        # Actually, for k=0: n0=(0,-1), so s0 = -y - R_WHITE. Above y = -R_WHITE => s0 > 0.
        # For each blade i in {0,1,2}, blade i is bounded by:
        #   - leading seam i (s_i <= 0 for the clockwise blade? Let's verify with concrete points!)
        # Test concrete points:
        #   Top (0, -0.7):   s0 = 0.7 - 0.475 > 0; s1 = (-0.7)*0.5 - 0.475 < 0; s2 = (-0.7)*0.5 - 0.475 < 0 -> ONLY s0 > 0? Wait!
        # Let's check tangent point of seam 2 (ang=240): n2 = (-0.866, 0.5), d2 = (-0.5, -0.866).
        # Ray 2 starts at R_WHITE * n2 = (-0.41, 0.237) and goes UP-LEFT in direction (-0.5, -0.866)!
        # So the TOP blade (0, -0.7) is between Ray 2 (top-left) and Ray 0 (top-right)!
        #   For (0, -0.7): s0 = -(-0.7) - 0.475 = +0.225 > 0 (above ray 0).
        #                  s2 = 0 + (-0.7)*0.5 - 0.475 = -0.825 < 0 (on the right/clockwise side of ray 2).
        #   Check (-0.7, -0.2) (upper-left, inside Top blade):
        #                  s0 = +0.2 - 0.475 = -0.275 < 0! Wait: (-0.7, -0.2) is BELOW y=-0.475,
        #                  so which ray separates Top from Bottom-Left? Ray 2!
        # Let's compute signed distance to Ray k = (t_along, d_perp):
        #   t_k = x*dx_k + y*dy_k  (along ray, > 0 means forward along the seam)
        #   p_k = x*nx_k + y*ny_k - R_WHITE (perpendicular: > 0 means CCW of ray k, < 0 means CW of ray k)
        # A point (x, y) with r > R_WHITE is strictly inside blade k if:
        #   - It is on the CW side of ray (k-1) OR CCW side of ray k... wait:
        #   Every point outside circle R_WHITE has a unique tangent ray angle!
        #   Specifically: angle from center to point is theta = atan2(y, x).
        #   Angle from center to tangent point of the ray passing through (x, y) is:
        #   phi = theta - acos(R_WHITE / r)   (in y-down CW coords!)
        #   Let's verify at (0.7, -0.475) on Ray 0:
        #     r = sqrt(0.49 + 0.2256) = 0.846
        #     theta = atan2(-0.475, 0.7) = -34.16 deg
        #     acos(0.475 / 0.846) = 55.84 deg
        #     theta - acos(...) = -90.0 deg -> EXACTLY tangent point (0, -R_WHITE)!!
        #   GENIUS!! That's the EXACT analytical formula for Chromium's 3 tangent blades!
        r = math.hypot(x, y)
        theta = math.atan2(y, x)
        alpha = math.acos(min(1.0, R_WHITE / max(r, R_WHITE)))
        # phi is the angle of the tangent point on the inner ring (in degrees, [-180, 180])
        phi = (math.degrees(theta - alpha) + 90.0) % 360.0
        # Seam 0 is at phi = 0 deg (top)
        # Seam 1 is at phi = 120 deg (bottom-right)
        # Seam 2 is at phi = 240 deg (bottom-left)
        # Wait: Moving CW from top (phi=0) to bottom-right (phi=120) is Blade 1 (right-bottom)!
        #       Moving CW from 120 to 240 is Blade 2 (bottom-left)!
        #       Moving CW from 240 to 360 is Blade 0 (top)!
        if phi < 120.0:
            blade = 1  # Right / Bottom-Right (Sky Cyan-Blue)
            seam_dist = phi  # degrees past seam 0
        elif phi < 240.0:
            blade = 2  # Bottom-Left (Indigo-Blue)
            seam_dist = phi - 120.0
        else:
            blade = 0  # Top (Royal Cobalt Blue)
            seam_dist = phi - 240.0
        return blade, seam_dist, r

    # Palette (LPBrower Chromium-core signature shades)
    # Blade 0 (Top): Deep Royal Cobalt -> Crisp Blue
    B0_A = (29, 78, 216)     # #1D4ED8
    B0_B = (37, 99, 235)     # #2563EB
    # Blade 1 (Bottom-Right): Luminous Cyan-Sky -> Ice Blue
    B1_A = (56, 189, 248)    # #38BDF8
    B1_B = (125, 211, 252)   # #7DD3FC
    # Blade 2 (Bottom-Left): Vibrant Azure-Indigo -> Blue
    B2_A = (79, 70, 229)     # #4F46E5
    B2_B = (59, 130, 246)    # #3B82F6

    CORE_TOP = (59, 130, 246)   # #3B82F6
    CORE_BOT = (29, 78, 216)    # #1D4ED8

    for iy in range(size):
        y = (iy - cy) / scale
        for ix in range(size):
            x = (ix - cx) / scale
            r = math.hypot(x, y)
            if r > R_OUT + AA:
                continue

            # Outer alpha antialiasing
            if r > R_OUT - AA:
                outer_a = max(0.0, min(1.0, (R_OUT + AA - r) / (2.0 * AA)))
            else:
                outer_a = 1.0

            # Compute ring/core color
            if r <= R_CORE - AA:
                # Inside center orb
                t_orb = (y + R_CORE) / (2.0 * R_CORE)
                cr, cg, cb = lerp_rgb(CORE_TOP, CORE_BOT, t_orb)
                # Subtle top-left specular glow
                spec = max(0.0, 1.0 - math.hypot(x + 0.12, y + 0.14) / 0.38)
                cr = min(255, int(cr + spec * 28))
                cg = min(255, int(cg + spec * 32))
                cb = min(255, int(cb + spec * 20))
                px[ix, iy] = (cr, cg, cb, 255)
            elif r <= R_CORE + AA:
                # AA transition Core -> White ring
                t = (r - (R_CORE - AA)) / (2.0 * AA)
                t_orb = (y + R_CORE) / (2.0 * R_CORE)
                cr, cg, cb = lerp_rgb(CORE_TOP, CORE_BOT, t_orb)
                col = lerp_rgb((cr, cg, cb), (255, 255, 255), t)
                px[ix, iy] = (col[0], col[1], col[2], 255)
            elif r <= R_WHITE - AA:
                # Pure crisp white ring
                px[ix, iy] = (255, 255, 255, 255)
            else:
                # Outer blades
                blade, seam_deg, _ = point_blade(x, y)
                # Smooth directional gradient per blade
                if blade == 0:
                    tg = (x + 0.9) / 1.8
                    br, bg, bb = lerp_rgb(B0_A, B0_B, tg)
                elif blade == 1:
                    tg = (y + 0.5) / 1.4
                    br, bg, bb = lerp_rgb(B1_A, B1_B, tg)
                else:
                    tg = (y + 0.4) / 1.3
                    br, bg, bb = lerp_rgb(B2_A, B2_B, tg)

                # Subtle fold shadow cast right past each tangent seam (0..18 deg)
                if seam_deg < 18.0:
                    sh = (1.0 - seam_deg / 18.0) ** 2 * 0.16
                    br = int(br * (1.0 - sh))
                    bg = int(bg * (1.0 - sh))
                    bb = int(bb * (1.0 - sh))

                # Subtle edge vignette for crisp depth
                if r > R_OUT - 0.08:
                    vig = ((r - (R_OUT - 0.08)) / 0.08) * 0.05
                    br = int(br * (1.0 - vig))
                    bg = int(bg * (1.0 - vig))
                    bb = int(bb * (1.0 - vig))

                # AA transition White ring -> Blade
                if r <= R_WHITE + AA:
                    t = (r - (R_WHITE - AA)) / (2.0 * AA)
                    col = lerp_rgb((255, 255, 255), (br, bg, bb), t)
                    px[ix, iy] = (col[0], col[1], col[2], 255)
                else:
                    px[ix, iy] = (br, bg, bb, int(outer_a * 255))

    if draw_lp:
        # Draw crisp, bold geometric white "LP" vector monogram centered inside the core orb!
        draw = ImageDraw.Draw(img)
        # Core orb diameter = 2 * R_CORE * scale; fit LP cleanly inside ~58% of orb diameter
        u = size / 1024.0
        # Center of LP block at (512, 512)
        # Total width ~ 236u, height ~ 188u, stroke ~ 38u
        ox = 512.0 - 116.0 * u
        oy = 512.0 - 94.0 * u
        st = 36.0 * u
        rad = 10.0 * u

        # Letter L: vertical stem + horizontal foot
        draw.rounded_rectangle(
            [ox, oy, ox + st, oy + 188 * u],
            radius=rad,
            fill=(255, 255, 255, 255),
        )
        draw.rounded_rectangle(
            [ox, oy + 188 * u - st, ox + 102 * u, oy + 188 * u],
            radius=rad,
            fill=(255, 255, 255, 255),
        )

        # Letter P: vertical stem + clean rounded bowl
        px0 = ox + 122 * u
        draw.rounded_rectangle(
            [px0, oy, px0 + st, oy + 188 * u],
            radius=rad,
            fill=(255, 255, 255, 255),
        )
        # P bowl outer rounded rect
        bowl_h = 114 * u
        bowl_w = 114 * u
        draw.rounded_rectangle(
            [px0, oy, px0 + bowl_w, oy + bowl_h],
            radius=bowl_h / 2.0,
            fill=(255, 255, 255, 255),
        )
        # P bowl inner cutout (matching center orb color at that y!)
        cut_x0 = px0 + st
        cut_y0 = oy + st * 0.85
        cut_x1 = px0 + bowl_w - st * 0.85
        cut_y1 = oy + bowl_h - st * 0.85
        # Sample orb mid-blue color for crisp bowl cutout
        mid_orb = lerp_rgb(CORE_TOP, CORE_BOT, 0.36)
        draw.rounded_rectangle(
            [cut_x0, cut_y0, cut_x1, cut_y1],
            radius=(cut_y1 - cut_y0) / 2.0,
            fill=(mid_orb[0], mid_orb[1], mid_orb[2], 255),
        )

    return img


def main():
    print("Rendering 1024x1024 supersampled Chromium-core LP browser icon...")
    hi_lp = render_chromium_lp_icon(1024, draw_lp=True)
    hi_clean = render_chromium_lp_icon(1024, draw_lp=False)

    # Save 512x512 master PNGs
    img512 = hi_lp.resize((512, 512), Image.Resampling.LANCZOS)
    img512.save(OUT_TAURI_PNG, "PNG")
    img512.save(OUT_ARTIFACT, "PNG")
    img512.save(os.path.join(OUT_PUBLIC, "lp-browser-512.png"), "PNG")

    # Save scaled PNGs (at 16px, pure orb without cramped text looks ultra-crisp!)
    for sz in (256, 128, 64, 48, 32):
        hi_lp.resize((sz, sz), Image.Resampling.LANCZOS).save(
            os.path.join(OUT_PUBLIC, f"lp-browser-{sz}.png"), "PNG"
        )
    hi_clean.resize((16, 16), Image.Resampling.LANCZOS).save(
        os.path.join(OUT_PUBLIC, "lp-browser-16.png"), "PNG"
    )

    # Build multi-resolution Windows ICO (16, 24, 32, 48, 64, 128, 256)
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    # Use Pillow's ICO writer with explicit crisp frames
    frames = []
    for sz in ico_sizes:
        src = hi_clean if sz <= 20 else hi_lp
        frames.append(src.resize((sz, sz), Image.Resampling.LANCZOS))
    frames[-1].save(
        OUT_TAURI_ICO,
        format="ICO",
        append_images=frames[:-1],
    )
    print(f"Saved {OUT_TAURI_PNG}, {OUT_TAURI_ICO}, and public/lp-browser-*.png")


if __name__ == "__main__":
    main()
