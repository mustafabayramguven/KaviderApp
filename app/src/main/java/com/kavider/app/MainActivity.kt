package com.kavider.app

import android.content.Context
import android.content.SharedPreferences
import android.os.Bundle
import android.view.KeyEvent
import android.view.Menu
import android.view.MenuItem
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.EditText
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.google.android.material.dialog.MaterialAlertDialogBuilder

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView
    private lateinit var prefs: SharedPreferences
    private val PREF_NAME = "kavider_prefs"
    private val KEY_URL = "target_url"
    private val KEY_FIRST = "is_first_launch"
    private val DEFAULT_URL = "https://askimbakbubiz.duckdns.org/"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        prefs = getSharedPreferences(PREF_NAME, Context.MODE_PRIVATE)
        setupFullScreen()

        val isFirst = prefs.getBoolean(KEY_FIRST, true)
        if (isFirst) {
            showUrlDialog(DEFAULT_URL, setup = true)
        } else {
            loadWebView(prefs.getString(KEY_URL, DEFAULT_URL) ?: DEFAULT_URL)
        }
    }

    private fun setupFullScreen() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).apply {
            hide(WindowInsetsCompat.Type.statusBars())
            hide(WindowInsetsCompat.Type.navigationBars())
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
        window.statusBarColor = android.graphics.Color.TRANSPARENT
        window.navigationBarColor = android.graphics.Color.TRANSPARENT
    }

    private fun loadWebView(url: String) {
        webView = WebView(this).apply {
            settings.apply {
                javaScriptEnabled = true
                domStorageEnabled = true
                loadWithOverviewMode = true
                useWideViewPort = true
                cacheMode = WebSettings.LOAD_CACHE_ELSE_NETWORK
                setSupportZoom(false)
                builtInZoomControls = false
                allowFileAccess = false
            }
            webViewClient = object : WebViewClient() {
                override fun shouldOverrideUrlLoading(view: WebView?, request: android.webkit.WebResourceRequest?) = false
            }
            loadUrl(url)
        }
        setContentView(webView)
        prefs.edit().putBoolean(KEY_FIRST, false).apply()
    }

    private fun showUrlDialog(current: String, setup: Boolean) {
        val input = EditText(this).apply {
            setText(current)
            hint = "https://ornek.com"
            setSingleLine()
        }
        MaterialAlertDialogBuilder(this)
            .setTitle(if (setup) "Kavider'e Hoş Geldin" else "Link Değiştir")
            .setMessage(if (setup) "Açılışta açılacak adresi gir:" else "Yeni adres gir:")
            .setView(input)
            .setPositiveButton("Tamam") { _, _ ->
                var newUrl = input.text.toString().trim()
                if (newUrl.isNotBlank()) {
                    if (!newUrl.startsWith("http://") && !newUrl.startsWith("https://")) newUrl = "https://$newUrl"
                    prefs.edit().putString(KEY_URL, newUrl).apply()
                    if (setup) loadWebView(newUrl) else webView.loadUrl(newUrl)
                }
            }
            .setCancelable(false)
            .show()
    }

    // Üst sağ köşede 3 nokta menüsü
    override fun onCreateOptionsMenu(menu: Menu?): Boolean {
        menu?.add(0, 1, 0, "Link Ayarla")?.setShowAsAction(MenuItem.SHOW_AS_ACTION_IF_ROOM)
        return true
    }

    override fun onOptionsItemSelected(item: MenuItem) = when (item.itemId) {
        1 -> {
            showUrlDialog(prefs.getString(KEY_URL, DEFAULT_URL) ?: DEFAULT_URL, setup = false)
            true
        }
        else -> super.onOptionsItemSelected(item)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        if (keyCode == KeyEvent.KEYCODE_BACK && ::webView.isInitialized && webView.canGoBack()) {
            webView.goBack()
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onPause() { if (::webView.isInitialized) webView.onPause(); super.onPause() }
    override fun onResume() { super.onResume(); if (::webView.isInitialized) webView.onResume() }
    override fun onDestroy() {
        if (::webView.isInitialized) webView.run { stopLoading(); clearHistory(); destroy() }
        super.onDestroy()
    }
}