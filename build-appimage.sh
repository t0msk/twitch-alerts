#!/bin/bash

cargo build --release

mkdir -p TwitchAlerts.AppDir/usr/share/icons/hicolor/256x256/apps
mkdir -p TwitchAlerts.AppDir/usr/bin

rm TwitchAlerts.AppDir/AppRun

cp target/release/twitch-alerts-rs TwitchAlerts.AppDir/usr/bin/
cp icon.png TwitchAlerts.AppDir/usr/share/icons/hicolor/256x256/apps/twitch-alerts.png

rm Twitch_Alerts-x86_64.AppImage

NO_STRIP=true linuxdeploy --appdir TwitchAlerts.AppDir --desktop-file twitch-alerts.desktop --icon-file icon.png --output appimage

