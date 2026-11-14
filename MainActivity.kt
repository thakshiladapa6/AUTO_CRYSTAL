package com.example.jarvis

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import android.widget.*

class MainActivity : Activity() {
    override fun onCreate(s: Bundle?) {
        super.onCreate(s)
        val p = Prefs.get(this)
        val l = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(48, 96, 48, 48) }
        fun btn(t: String, f: () -> Unit) { l.addView(Button(this).apply { text = t; setOnClickListener { f() } }) }
        fun field(h: String, k: String): EditText {
            val e = EditText(this).apply { hint = h; setText(p.getString(k, "")) }
            l.addView(e); return e
        }
        l.addView(TextView(this).apply { text = "Jarvis setup (do once)"; textSize = 24f })
        btn("1. Allow microphone & notifications") {
            requestPermissions(arrayOf(Manifest.permission.RECORD_AUDIO, Manifest.permission.POST_NOTIFICATIONS), 1)
        }
        btn("2. Enable Accessibility service") { startActivity(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)) }
        btn("3. Allow display over other apps") {
            startActivity(Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION, Uri.parse("package:$packageName")))
        }
        btn("4. Disable battery optimization") { startActivity(Intent(Settings.ACTION_IGNORE_BATTERY_OPTIMIZATION_SETTINGS)) }
        val llm = field("OpenRouter API key (sk-or-...)", "llm_key")
        val model = field("Model (blank = anthropic/claude-sonnet-4.5)", "model")
        val pico = field("Picovoice access key", "pico")
        btn("5. Save & start Jarvis") {
            p.edit().putString("llm_key", llm.text.toString().trim())
                .putString("model", model.text.toString().trim())
                .putString("pico", pico.text.toString().trim()).apply()
            startForegroundService(Intent(this, WakeWordService::class.java))
            Toast.makeText(this, "Done. Just say: Jarvis", Toast.LENGTH_LONG).show()
        }
        setContentView(ScrollView(this).apply { addView(l) })
    }
}
