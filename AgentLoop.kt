package com.example.jarvis

import android.content.Context
import android.content.Intent
import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/** Observe screen -> ask Claude for one action -> execute -> repeat until "done". Run on a background thread. */
object AgentLoop {
    private const val DEFAULT_MODEL = "anthropic/claude-sonnet-4.5"
    private const val SYSTEM = """You control an Android phone for the user. Each turn you get the current screen as an indexed list. Reply with ONE JSON object only, no other text. Actions:
{"action":"open_app","name":"app name"}
{"action":"click","id":N}
{"action":"type","id":N,"text":"..."}
{"action":"scroll","dir":"down"|"up"}
{"action":"global","name":"home"|"back"|"recents"|"notifications"|"quick_settings"}
{"action":"wait"}
{"action":"speak","text":"..."}
{"action":"done","text":"short spoken answer to the user"}
Rules: be efficient. Before sending messages, making payments, deleting data or changing security settings, do NOT do it: use done to ask the user to confirm and wait for their next command. Never enter passwords, OTPs or card numbers; use done to tell the user to do it."""

    fun run(ctx: Context, command: String, speak: (String) -> Unit) {
        val prefs = Prefs.get(ctx)
        val key = prefs.getString("llm_key", "") ?: ""
        val model = prefs.getString("model", "")?.ifBlank { null } ?: DEFAULT_MODEL
        val msgs = JSONArray()
        var last = "start"
        try {
            repeat(15) { step ->
                val svc = JarvisAccessibilityService.instance
                if (svc == null) { speak("Please enable the accessibility service first."); return }
                val screen = svc.dump()
                val text = (if (step == 0) "User command: $command\n" else "") +
                    "Last action result: $last\nScreen:\n$screen"
                msgs.put(JSONObject().put("role", "user").put("content", text))
                val reply = call(key, model, msgs)
                msgs.put(JSONObject().put("role", "assistant").put("content", reply))
                val a = JSONObject(reply.substring(reply.indexOf('{'), reply.lastIndexOf('}') + 1))
                last = when (a.getString("action")) {
                    "open_app" -> openApp(svc, a.getString("name")).toString()
                    "click" -> svc.click(a.getInt("id")).toString()
                    "type" -> svc.type(a.getInt("id"), a.getString("text")).toString()
                    "scroll" -> { svc.scroll(a.optString("dir") == "down"); "ok" }
                    "global" -> svc.global(a.getString("name")).toString()
                    "speak" -> { speak(a.optString("text")); "ok" }
                    "done" -> { speak(a.optString("text", "Done")); return }
                    else -> "ok"
                }
                Thread.sleep(1200)
            }
            speak("I couldn't finish that in time.")
        } catch (e: Exception) {
            speak("Something went wrong: ${e.message}")
        }
    }

    private fun openApp(svc: Context, name: String): Boolean {
        val pm = svc.packageManager
        val app = pm.getInstalledApplications(0)
            .firstOrNull { pm.getApplicationLabel(it).toString().contains(name, true) } ?: return false
        val i = pm.getLaunchIntentForPackage(app.packageName) ?: return false
        i.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        svc.startActivity(i); return true
    }

    private fun call(key: String, model: String, history: JSONArray): String {
        val msgs = JSONArray().put(JSONObject().put("role", "system").put("content", SYSTEM))
        for (i in 0 until history.length()) msgs.put(history.get(i))
        val c = URL("https://openrouter.ai/api/v1/chat/completions").openConnection() as HttpURLConnection
        c.requestMethod = "POST"; c.doOutput = true
        c.setRequestProperty("Authorization", "Bearer $key")
        c.setRequestProperty("Content-Type", "application/json")
        c.setRequestProperty("X-Title", "Jarvis")
        val body = JSONObject().put("model", model).put("max_tokens", 300).put("messages", msgs)
        c.outputStream.use { it.write(body.toString().toByteArray()) }
        val s = if (c.responseCode < 400) c.inputStream else c.errorStream
        val json = JSONObject(s.bufferedReader().readText())
        if (json.has("error")) throw Exception(json.getJSONObject("error").optString("message", "API error"))
        return json.getJSONArray("choices").getJSONObject(0).getJSONObject("message").getString("content")
    }
}
