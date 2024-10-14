#!/bin/sh -e

. ../../common-script.sh

installMeld() {
    if ! command_exists meld; then
        printf "%b\n" "${YELLOW}Installing Meld...${RC}"
        case "$PACKAGER" in
            pacman)
                elevated_execution "$PACKAGER" -S --needed --noconfirm meld
                ;;
            apt-get|nala)
                elevated_execution "$PACKAGER" -y install meld
                ;;
            *)
                . ../setup-flatpak.sh
                flatpak install -y flathub org.gnome.meld
                ;;
        esac
    else
        printf "%b\n" "${GREEN}Meld is already installed.${RC}"
    fi
}

checkEnv
checkEscalationTool
installMeld