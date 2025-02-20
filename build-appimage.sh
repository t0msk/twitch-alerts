#!/bin/bash

rm TwitchAlerts.AppDir/AppRun

NO_STRIP=true linuxdeploy --appdir TwitchAlerts.AppDir --desktop-file twitch-alerts.desktop --icon-file icon.png --output appimage

