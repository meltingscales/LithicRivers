package com.meltingscales.lithicrivers;

import android.os.Bundle;
import android.view.KeyEvent;

import androidx.activity.OnBackPressedCallback;

import com.google.androidgamesdk.GameActivity;

/**
 * GameActivity finishes itself on Back, which would end the game mid-session. Back is
 * the game's Escape instead (close a window, open the pause menu); Home leaves the app.
 */
public class MainActivity extends GameActivity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getOnBackPressedDispatcher().addCallback(this, new OnBackPressedCallback(true) {
            @Override
            public void handleOnBackPressed() {
                onKeyDown(KeyEvent.KEYCODE_ESCAPE, new KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_ESCAPE));
                onKeyUp(KeyEvent.KEYCODE_ESCAPE, new KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_ESCAPE));
            }
        });
    }
}
