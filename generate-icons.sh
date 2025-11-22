#!/bin/bash

# Скрипт для генерации иконок приложения
# Требуется ImageMagick: sudo apt install imagemagick (Linux) или brew install imagemagick (macOS)

echo "Generating application icons..."

# Создаем директорию для иконок
mkdir -p src-tauri/icons

# Создаем базовую SVG иконку
cat > src-tauri/icons/icon.svg << 'EOF'
<svg width="512" height="512" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#ff6b35;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#f7931e;stop-opacity:1" />
    </linearGradient>
  </defs>
  
  <!-- Background -->
  <rect width="512" height="512" rx="80" fill="url(#grad)"/>
  
  <!-- Rust gear symbol -->
  <g transform="translate(256,256)">
    <!-- Outer gear -->
    <circle cx="0" cy="0" r="150" fill="#1a1a1a" opacity="0.9"/>
    
    <!-- Inner circle -->
    <circle cx="0" cy="0" r="90" fill="white"/>
    
    <!-- Gear teeth -->
    <path d="M 0,-150 L 15,-165 L -15,-165 Z M 150,0 L 165,15 L 165,-15 Z M 0,150 L -15,165 L 15,165 Z M -150,0 L -165,-15 L -165,15 Z" fill="#1a1a1a"/>
    
    <!-- Rust logo simplified -->
    <text x="0" y="15" font-family="Arial" font-size="120" font-weight="bold" text-anchor="middle" fill="#ff6b35">R</text>
  </g>
  
  <!-- Graph nodes representation -->
  <circle cx="100" cy="100" r="20" fill="white" opacity="0.7"/>
  <circle cx="412" cy="100" r="20" fill="white" opacity="0.7"/>
  <circle cx="100" cy="412" r="20" fill="white" opacity="0.7"/>
  <circle cx="412" cy="412" r="20" fill="white" opacity="0.7"/>
  
  <!-- Connecting lines -->
  <line x1="100" y1="100" x2="412" y2="100" stroke="white" stroke-width="4" opacity="0.5"/>
  <line x1="100" y1="100" x2="100" y2="412" stroke="white" stroke-width="4" opacity="0.5"/>
  <line x1="412" y1="100" x2="412" y2="412" stroke="white" stroke-width="4" opacity="0.5"/>
  <line x1="100" y1="412" x2="412" y2="412" stroke="white" stroke-width="4" opacity="0.5"/>
</svg>
EOF

# Проверяем наличие ImageMagick
if ! command -v convert &> /dev/null; then
    echo "ImageMagick not found. Please install it:"
    echo "  Ubuntu/Debian: sudo apt install imagemagick"
    echo "  macOS: brew install imagemagick"
    echo "  Windows: Download from https://imagemagick.org/script/download.php"
    exit 1
fi

# Генерируем PNG иконки разных размеров
convert src-tauri/icons/icon.svg -resize 32x32 src-tauri/icons/32x32.png
convert src-tauri/icons/icon.svg -resize 128x128 src-tauri/icons/128x128.png
convert src-tauri/icons/icon.svg -resize 256x256 src-tauri/icons/128x128@2x.png
convert src-tauri/icons/icon.svg -resize 512x512 src-tauri/icons/icon.png

# Для macOS (icns)
if command -v iconutil &> /dev/null; then
    mkdir -p src-tauri/icons/icon.iconset
    convert src-tauri/icons/icon.svg -resize 16x16 src-tauri/icons/icon.iconset/icon_16x16.png
    convert src-tauri/icons/icon.svg -resize 32x32 src-tauri/icons/icon.iconset/icon_16x16@2x.png
    convert src-tauri/icons/icon.svg -resize 32x32 src-tauri/icons/icon.iconset/icon_32x32.png
    convert src-tauri/icons/icon.svg -resize 64x64 src-tauri/icons/icon.iconset/icon_32x32@2x.png
    convert src-tauri/icons/icon.svg -resize 128x128 src-tauri/icons/icon.iconset/icon_128x128.png
    convert src-tauri/icons/icon.svg -resize 256x256 src-tauri/icons/icon.iconset/icon_128x128@2x.png
    convert src-tauri/icons/icon.svg -resize 256x256 src-tauri/icons/icon.iconset/icon_256x256.png
    convert src-tauri/icons/icon.svg -resize 512x512 src-tauri/icons/icon.iconset/icon_256x256@2x.png
    convert src-tauri/icons/icon.svg -resize 512x512 src-tauri/icons/icon.iconset/icon_512x512.png
    convert src-tauri/icons/icon.svg -resize 1024x1024 src-tauri/icons/icon.iconset/icon_512x512@2x.png
    iconutil -c icns src-tauri/icons/icon.iconset -o src-tauri/icons/icon.icns
    rm -rf src-tauri/icons/icon.iconset
fi

# Для Windows (ico) - требуется ImageMagick с поддержкой ICO
convert src-tauri/icons/icon.svg -define icon:auto-resize=256,128,64,48,32,16 src-tauri/icons/icon.ico

echo "Icons generated successfully in src-tauri/icons/"
