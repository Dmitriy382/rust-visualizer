#!/bin/bash

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🦀 Rust Project Visualizer - Setup Script${NC}\n"

# Проверка Node.js
echo -e "${YELLOW}Checking Node.js...${NC}"
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js не установлен. Пожалуйста, установите Node.js 18+ с https://nodejs.org/${NC}"
    exit 1
fi
NODE_VERSION=$(node -v)
echo -e "${GREEN}✓ Node.js ${NODE_VERSION} найден${NC}"

# Проверка Rust
echo -e "${YELLOW}Checking Rust...${NC}"
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}❌ Rust не установлен.${NC}"
    echo -e "${YELLOW}Хотите установить Rust? (y/n)${NC}"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source $HOME/.cargo/env
    else
        exit 1
    fi
fi
RUST_VERSION=$(rustc --version)
echo -e "${GREEN}✓ ${RUST_VERSION} найден${NC}"

# Проверка системных зависимостей (Linux)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${YELLOW}Checking system dependencies...${NC}"
    
    MISSING_DEPS=()
    
    if ! dpkg -l | grep -q libwebkit2gtk-4.0-dev; then
        MISSING_DEPS+=("libwebkit2gtk-4.0-dev")
    fi
    
    if ! dpkg -l | grep -q libgtk-3-dev; then
        MISSING_DEPS+=("libgtk-3-dev")
    fi
    
    if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
        echo -e "${YELLOW}Требуется установка системных зависимостей:${NC}"
        echo "${MISSING_DEPS[@]}"
        echo -e "${YELLOW}Установить? (требуется sudo) (y/n)${NC}"
        read -r response
        if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
            sudo apt update
            sudo apt install -y libwebkit2gtk-4.0-dev build-essential curl wget \
                libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
        fi
    else
        echo -e "${GREEN}✓ Все системные зависимости установлены${NC}"
    fi
fi

# Создание структуры директорий
echo -e "\n${YELLOW}Creating project structure...${NC}"

mkdir -p src/components
mkdir -p src-tauri/src

echo -e "${GREEN}✓ Структура директорий создана${NC}"

# Установка npm зависимостей
echo -e "\n${YELLOW}Installing npm dependencies...${NC}"
npm install

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ npm зависимости установлены${NC}"
else
    echo -e "${RED}❌ Ошибка установки npm зависимостей${NC}"
    exit 1
fi

# Сборка Rust зависимостей
echo -e "\n${YELLOW}Building Rust dependencies (это может занять несколько минут)...${NC}"
cd src-tauri
cargo build

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Rust зависимости собраны${NC}"
else
    echo -e "${RED}❌ Ошибка сборки Rust зависимостей${NC}"
    exit 1
fi

cd ..

echo -e "\n${GREEN}✅ Установка завершена успешно!${NC}"
echo -e "\n${YELLOW}Запустите приложение командой:${NC}"
echo -e "${GREEN}npm run tauri:dev${NC}"
