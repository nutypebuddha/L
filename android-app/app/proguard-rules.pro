# Keep JavaScript interface
-keepclassmembers class ai.lai.assistant.MainActivity$LaiBridge {
    @android.webkit.JavascriptInterface <methods>;
}

# Keep WebView
-keep class android.webkit.** { *; }

# Keep org.json (used by MCP response parsing)
-keep class org.json.** { *; }

# Keep model download classes
-keep class java.net.** { *; }
-keep class java.io.FileOutputStream { *; }
