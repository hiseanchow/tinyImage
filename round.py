from PIL import Image, ImageDraw

def mask_image(img_path, output_path, radius_ratio=0.225):
    img = Image.open(img_path).convert("RGBA")
    w, h = img.size
    
    # 4x supersampling for ultra-smooth anti-aliased edges
    scale = 4
    mask = Image.new("L", (w * scale, h * scale), 0)
    draw = ImageDraw.Draw(mask)
    
    radius = int(min(w, h) * radius_ratio * scale)
    draw.rounded_rectangle((0, 0, w * scale - 1, h * scale - 1), radius=radius, fill=255)
    
    mask = mask.resize((w, h), Image.Resampling.LANCZOS)
    
    # Apply mask
    result = img.copy()
    result.putalpha(mask)
    
    # Save back
    result.save(output_path)
    print(f"Masked icon effectively matching macOS squircle shape.")

if __name__ == "__main__":
    icon_path = "src-tauri/icons/icon.png"
    mask_image(icon_path, icon_path)
