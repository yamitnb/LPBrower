import os
from PIL import Image, ImageDraw, ImageFilter, ImageFont

def create_master_lp_icon(size=1024):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    cx, cy = size // 2, size // 2

    # 1. Background rounded squircle
    pad = int(size * 0.04)
    corner_radius = int(size * 0.22)
    
    bg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg)
    # Deep Cyber Dark gradient simulation
    bg_draw.rounded_rectangle([pad, pad, size - pad, size - pad], radius=corner_radius, fill=(11, 15, 25, 255))
    
    # Outer subtle rim border
    border_color = (99, 102, 241, 160) # Indigo rim
    bg_draw.rounded_rectangle([pad, pad, size - pad, size - pad], radius=corner_radius, outline=border_color, width=max(2, int(size * 0.008)))
    
    # Radial cyber glow in center
    glow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    glow_draw.ellipse([cx - int(size * 0.35), cy - int(size * 0.35), cx + int(size * 0.35), cy + int(size * 0.35)], fill=(99, 102, 241, 70))
    glow_draw.ellipse([cx - int(size * 0.20), cy - int(size * 0.20), cx + int(size * 0.20), cy + int(size * 0.20)], fill=(6, 182, 212, 100))
    glow = glow.filter(ImageFilter.GaussianBlur(radius=int(size * 0.08)))
    
    img.alpha_composite(bg)
    img.alpha_composite(glow)
    
    # 2. Draw Vector LP Elements
    fg = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    fg_draw = ImageDraw.Draw(fg)

    # Geometry for "L" and "P"
    l_left = cx - int(size * 0.28)
    l_width = int(size * 0.11)
    l_top = cy - int(size * 0.28)
    l_bottom = cy + int(size * 0.26)
    l_base_right = cx - int(size * 0.02)

    # Draw L with cyan-blue gradient
    for y in range(l_top, l_bottom + 1):
        t = (y - l_top) / max(1, (l_bottom - l_top))
        # Gradient from Cyan (0, 242, 254) to Cobalt (79, 70, 229)
        r = int(0 * (1 - t) + 79 * t)
        g = int(242 * (1 - t) + 70 * t)
        b = int(254 * (1 - t) + 229 * t)
        
        if y < l_bottom - l_width:
            x1 = l_left
            x2 = l_left + l_width
            fg_draw.line([(x1, y), (x2, y)], fill=(r, g, b, 255), width=1)
        else:
            x1 = l_left
            x2 = l_base_right
            fg_draw.line([(x1, y), (x2, y)], fill=(r, g, b, 255), width=1)
            
    # Rounded corners for L
    r_cap = l_width // 2
    fg_draw.ellipse([l_left, l_top - r_cap, l_left + l_width, l_top + r_cap], fill=(0, 242, 254, 255))
    fg_draw.ellipse([l_base_right - r_cap, l_bottom - l_width, l_base_right + r_cap, l_bottom], fill=(79, 70, 229, 255))

    # P: Vertical stem and curved browser loop
    p_left = cx - int(size * 0.06)
    p_width = int(size * 0.11)
    p_top = cy - int(size * 0.28)
    p_bottom = cy + int(size * 0.26)
    
    # Draw P Stem
    for y in range(p_top, p_bottom + 1):
        t = (y - p_top) / max(1, (p_bottom - p_top))
        # Gradient from Indigo (99, 102, 241) to Violet (168, 85, 247)
        r = int(99 * (1 - t) + 168 * t)
        g = int(102 * (1 - t) + 85 * t)
        b = int(241 * (1 - t) + 247 * t)
        fg_draw.line([(p_left, y), (p_left + p_width, y)], fill=(r, g, b, 255), width=1)
        
    p_r_cap = p_width // 2
    fg_draw.ellipse([p_left, p_bottom - p_r_cap, p_left + p_width, p_bottom + p_r_cap], fill=(168, 85, 247, 255))
    fg_draw.ellipse([p_left, p_top - p_r_cap, p_left + p_width, p_top + p_r_cap], fill=(99, 102, 241, 255))

    # Draw P Loop (The Browser Arc)
    p_loop_cy = p_top + int(size * 0.15)
    p_loop_cx = p_left + p_width
    p_outer_r = int(size * 0.16)
    p_inner_r = int(size * 0.06)
    
    # Outer arc of P
    loop_box = [p_loop_cx - p_outer_r, p_loop_cy - p_outer_r, p_loop_cx + p_outer_r, p_loop_cy + p_outer_r]
    fg_draw.pieslice(loop_box, -90, 90, fill=(129, 140, 248, 255))
    # Cut out inner hole
    inner_box = [p_loop_cx - p_inner_r, p_loop_cy - p_inner_r, p_loop_cx + p_inner_r, p_loop_cy + p_inner_r]
    fg_draw.ellipse(inner_box, fill=(11, 15, 25, 255))

    # Inner Identity Core (Radar / Shield Beacon)
    core_cx = p_loop_cx
    core_cy = p_loop_cy
    orbit_r = int(size * 0.038)
    orbit_box = [core_cx - orbit_r, core_cy - orbit_r, core_cx + orbit_r, core_cy + orbit_r]
    fg_draw.ellipse(orbit_box, outline=(6, 182, 212, 220), width=max(2, int(size * 0.007)))
    dot_r = int(size * 0.016)
    fg_draw.ellipse([core_cx - dot_r, core_cy - dot_r, core_cx + dot_r, core_cy + dot_r], fill=(0, 242, 254, 255))

    img.alpha_composite(fg)
    return img

if __name__ == "__main__":
    out_dir = r"g:\Vibe Tool\LPBrower\public"
    tauri_icons_dir = r"g:\Vibe Tool\LPBrower\src-tauri\icons"
    os.makedirs(out_dir, exist_ok=True)
    os.makedirs(tauri_icons_dir, exist_ok=True)
    
    master = create_master_lp_icon(1024)
    
    # Save various sizes
    for s in [512, 256, 128, 64, 32, 16]:
        resized = master.resize((s, s), Image.Resampling.LANCZOS)
        resized.save(os.path.join(out_dir, f"lp-icon-{s}.png"))
        
    # Save into src-tauri/icons
    master.resize((512, 512), Image.Resampling.LANCZOS).save(os.path.join(tauri_icons_dir, "icon.png"))
    master.resize((128, 128), Image.Resampling.LANCZOS).save(os.path.join(tauri_icons_dir, "128x128.png"))
    master.resize((256, 256), Image.Resampling.LANCZOS).save(os.path.join(tauri_icons_dir, "128x128@2x.png"))
    master.resize((64, 64), Image.Resampling.LANCZOS).save(os.path.join(tauri_icons_dir, "64x64.png"))
    master.resize((32, 32), Image.Resampling.LANCZOS).save(os.path.join(tauri_icons_dir, "32x32.png"))
    
    # Windows tiles
    for tile_name, sz in [
        ("Square30x30Logo.png", 30),
        ("Square44x44Logo.png", 44),
        ("Square71x71Logo.png", 71),
        ("Square89x89Logo.png", 89),
        ("Square107x107Logo.png", 107),
        ("Square142x142Logo.png", 142),
        ("Square150x150Logo.png", 150),
        ("Square284x284Logo.png", 284),
        ("Square310x310Logo.png", 310),
        ("StoreLogo.png", 50),
    ]:
        master.resize((sz, sz), Image.Resampling.LANCZOS).save(os.path.join(tauri_icons_dir, tile_name))

    # Multi-resolution ICO files
    master.save(
        os.path.join(out_dir, "favicon.ico"),
        format="ICO",
        sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )
    master.save(
        os.path.join(tauri_icons_dir, "icon.ico"),
        format="ICO",
        sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )
    print("All LPBrower icons generated successfully!")
