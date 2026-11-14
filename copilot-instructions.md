# Project: Jarvis – voice-controlled phone agent (Android)

Build a native Android app (Kotlin, Jetpack Compose, minSdk 26, targetSdk 34+, Gradle Kotlin DSL, package `com.example.jarvis`). The repo must contain a working `gradlew` so GitHub Actions can run `./gradlew assembleDebug`.

## User experience
1. **One-time setup wizard** (first launch only). Steps, each with an explanation and a button that opens the right system screen:
   - Microphone permission (`RECORD_AUDIO`)
   - Notifications permission (`POST_NOTIFICATIONS`)
   - Enable **Accessibility Service** (Settings.ACTION_ACCESSIBILITY_SETTINGS)
   - Display over other apps (`SYSTEM_ALERT_WINDOW`)
   - Notification listener access
   - Contacts, Phone, SMS, Calendar permissions
   - Ignore battery optimizations (`REQUEST_IGNORE_BATTERY_OPTIMIZATIONS`)
   - Enter API keys: LLM API key (Anthropic Claude) and Picovoice access key. Store in `EncryptedSharedPreferences`.
2. After setup, the user only says **"Jarvis"**, then a command. No UI needed.
3. Jarvis replies with a short spoken answer (Android TextToSpeech) and a small overlay showing what it is doing, with a **Stop** button.

## Architecture
- `WakeWordService`: foreground service (type `microphone`, persistent notification) running Picovoice Porcupine with a custom "Jarvis" keyword. Restart on boot (`BOOT_COMPLETED` receiver).
- `SpeechInput`: after the wake word, record the command with `SpeechRecognizer` (or Whisper API as a fallback), with end-of-speech detection.
- `AgentLoop`: sends the command plus current screen state to the LLM using tool calling, executes the returned action, observes the new screen, and repeats until the LLM returns `done` (max 25 steps).
- `JarvisAccessibilityService` (extends `AccessibilityService`): the hands and eyes.
  - Dump the active window's node tree to compact JSON (text, contentDescription, viewId, bounds, clickable, editable, scrollable).
  - Actions: click node, long click, set text, scroll forward/back, swipe/tap by coordinates via `dispatchGesture`, `GLOBAL_ACTION_HOME/BACK/RECENTS/NOTIFICATIONS/QUICK_SETTINGS/LOCK_SCREEN/TAKE_SCREENSHOT`.
  - `takeScreenshot()` (API 30+) for vision when the node tree is not enough.
- `IntentActions` (fast paths that don't need screen control): launch app by name, call, SMS, alarm/timer, open URL, maps navigation, flashlight, volume, brightness, DND, Wi-Fi/Bluetooth settings panels, music control via `MediaSessionManager`.
- `LlmClient`: calls the Anthropic Messages API with tools: `open_app`, `click`, `type_text`, `scroll`, `swipe`, `press_global`, `call_contact`, `send_sms`, `set_alarm`, `speak`, `ask_user`, `done`.

## Safety (required)
- Ask for spoken confirmation before: sending messages/emails, making calls to non-favorites, purchases or payments, deleting data, changing security settings, factory reset.
- Never read out or type passwords, OTPs, or card numbers. Pause when a password field is focused and hand control back to the user.
- Hard kill switch: Stop button in the overlay, plus saying "Jarvis stop".
- Log every action locally so the user can review what was done.

## Build and CI
- `.github/workflows/build-apk.yml` builds the debug APK on every push.
- Secrets are read from `local.properties` / environment, never committed.

## Known limits to handle gracefully
- Android 14+ requires the foreground service type `microphone`.
- Some apps block accessibility reads (`FLAG_SECURE`, banking apps). Tell the user instead of failing silently.
- Accessibility-based agents may not be accepted on Google Play; target sideloading first.
- Always-on wake word costs battery; use Porcupine's low-power mode.

## Milestones
1. Project skeleton + setup wizard
2. Accessibility service + screen dump + basic actions
3. Wake word service + speech input + TTS
4. LLM agent loop + intent fast paths
5. Overlay, confirmations, kill switch
6. Polish, boot receiver, battery tuning
