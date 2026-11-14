package com.example.jarvis

import ai.picovoice.porcupine.Porcupine
import ai.picovoice.porcupine.PorcupineManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Bundle
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import android.speech.tts.TextToSpeech
import java.util.Locale

class WakeWordService : Service() {
    private var porcupine: PorcupineManager? = null
    private var tts: TextToSpeech? = null
    private val main = Handler(Looper.getMainLooper())

    override fun onBind(i: Intent?): IBinder? = null

    override fun onStartCommand(i: Intent?, f: Int, id: Int): Int {
        val ch = "jarvis"
        getSystemService(NotificationManager::class.java)
            .createNotificationChannel(NotificationChannel(ch, "Jarvis", NotificationManager.IMPORTANCE_LOW))
        val n = Notification.Builder(this, ch).setContentTitle("Jarvis is listening")
            .setSmallIcon(android.R.drawable.ic_btn_speak_now).build()
        startForeground(1, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE)
        tts = TextToSpeech(this) { tts?.language = Locale.getDefault() }
        startWake()
        return START_STICKY
    }

    private fun startWake() {
        val key = Prefs.get(this).getString("pico", "") ?: ""
        porcupine = PorcupineManager.Builder()
            .setAccessKey(key)
            .setKeyword(Porcupine.BuiltInKeyword.JARVIS)
            .build(this) { main.post { porcupine?.stop(); listen() } }
        porcupine?.start()
    }

    private fun speak(t: String) { tts?.speak(t, TextToSpeech.QUEUE_ADD, null, "j") }

    private fun listen() {
        speak("Yes?")
        main.postDelayed({
            val sr = SpeechRecognizer.createSpeechRecognizer(this)
            sr.setRecognitionListener(object : RecognitionListener {
                override fun onResults(b: Bundle?) {
                    val t = b?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)?.firstOrNull()
                    sr.destroy()
                    if (t == null) { porcupine?.start(); return }
                    Thread {
                        AgentLoop.run(this@WakeWordService, t) { s -> main.post { speak(s) } }
                        main.post { porcupine?.start() }
                    }.start()
                }
                override fun onError(e: Int) { sr.destroy(); porcupine?.start() }
                override fun onReadyForSpeech(p: Bundle?) {}
                override fun onBeginningOfSpeech() {}
                override fun onRmsChanged(v: Float) {}
                override fun onBufferReceived(b: ByteArray?) {}
                override fun onEndOfSpeech() {}
                override fun onPartialResults(b: Bundle?) {}
                override fun onEvent(t: Int, b: Bundle?) {}
            })
            sr.startListening(
                Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH)
                    .putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
            )
        }, 1200)
    }

    override fun onDestroy() { porcupine?.delete(); tts?.shutdown(); super.onDestroy() }
}
