package com.example.jarvis

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.os.Bundle
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class JarvisAccessibilityService : AccessibilityService() {
    companion object { @Volatile var instance: JarvisAccessibilityService? = null }

    private var nodes = listOf<AccessibilityNodeInfo>()

    override fun onServiceConnected() { instance = this }
    override fun onDestroy() { instance = null; super.onDestroy() }
    override fun onAccessibilityEvent(e: AccessibilityEvent?) {}
    override fun onInterrupt() {}

    /** Compact text description of the current screen; each useful element gets an index. */
    fun dump(): String {
        val root = rootInActiveWindow ?: return "(no screen available)"
        val list = mutableListOf<AccessibilityNodeInfo>()
        val sb = StringBuilder()
        fun walk(n: AccessibilityNodeInfo) {
            val t = n.text?.toString() ?: ""
            val d = n.contentDescription?.toString() ?: ""
            if (n.isClickable || n.isEditable || t.isNotEmpty() || d.isNotEmpty()) {
                sb.append("[${list.size}] $t $d")
                if (n.isClickable) sb.append(" (click)")
                if (n.isEditable) sb.append(" (edit)")
                sb.append("\n"); list.add(n)
            }
            for (i in 0 until n.childCount) n.getChild(i)?.let { walk(it) }
        }
        walk(root); nodes = list
        return sb.toString().take(6000)
    }

    fun click(i: Int): Boolean {
        var n: AccessibilityNodeInfo? = nodes.getOrNull(i)
        while (n != null && !n.isClickable) n = n.parent
        return n?.performAction(AccessibilityNodeInfo.ACTION_CLICK) == true
    }

    fun type(i: Int, text: String): Boolean {
        val n = nodes.getOrNull(i) ?: return false
        val b = Bundle()
        b.putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, text)
        return n.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, b)
    }

    fun scroll(down: Boolean) {
        val m = resources.displayMetrics
        val x = m.widthPixels / 2f
        val y1 = m.heightPixels * (if (down) 0.7f else 0.3f)
        val y2 = m.heightPixels * (if (down) 0.3f else 0.7f)
        val path = Path().apply { moveTo(x, y1); lineTo(x, y2) }
        dispatchGesture(
            GestureDescription.Builder()
                .addStroke(GestureDescription.StrokeDescription(path, 0, 300)).build(), null, null
        )
    }

    fun global(name: String): Boolean {
        val a = when (name) {
            "home" -> GLOBAL_ACTION_HOME
            "back" -> GLOBAL_ACTION_BACK
            "recents" -> GLOBAL_ACTION_RECENTS
            "notifications" -> GLOBAL_ACTION_NOTIFICATIONS
            "quick_settings" -> GLOBAL_ACTION_QUICK_SETTINGS
            else -> return false
        }
        return performGlobalAction(a)
    }
}
