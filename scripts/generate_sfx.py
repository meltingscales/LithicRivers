#!/usr/bin/env python3
"""Generate LithicRivers' sound effects as small WAV files.

Each sound is built from seeded noise and tones with a short envelope, so the output is
byte-identical every run and the sounds are the game's own. Tune a sound by editing its
function and rerunning:

    python3 scripts/generate_sfx.py

Writes 22.05 kHz mono 16-bit WAVs into crates/client/assets/sound/effects/.
"""

import math
import os
import random
import struct
import wave

RATE = 22050
OUT = os.path.join(os.path.dirname(__file__), "..", "crates", "client", "assets", "sound", "effects")


def env(t, length, attack=0.005, curve=4.0):
    """A quick attack, then an exponential-ish decay to silence at `length`."""
    if t < attack:
        return t / attack
    x = (t - attack) / max(length - attack, 1e-6)
    return max(0.0, (1.0 - x)) ** curve


def lowpass(samples, alpha):
    out, y = [], 0.0
    for s in samples:
        y += alpha * (s - y)
        out.append(y)
    return out


def render(length, voice):
    n = int(RATE * length)
    return [voice(i / RATE) for i in range(n)]


def noise(seed):
    rng = random.Random(seed)
    return lambda: rng.uniform(-1.0, 1.0)


def mine():
    """A gritty burst of stone with a low knock under it."""
    grit = noise(0x5701E)
    raw = render(0.16, lambda t: grit() * env(t, 0.16, curve=3.0))
    gritty = lowpass(raw, 0.35)
    knock = render(0.16, lambda t: math.sin(2 * math.pi * (110 - 200 * t) * t) * env(t, 0.12, curve=5.0))
    return [0.7 * g + 0.5 * k for g, k in zip(gritty, knock)]


def place():
    """A soft, low thump."""
    thud = noise(0x91ACE)
    body = render(0.12, lambda t: math.sin(2 * math.pi * (90 - 150 * t) * t) * env(t, 0.12, curve=6.0))
    dust = lowpass(render(0.12, lambda t: thud() * env(t, 0.05, curve=4.0)), 0.15)
    return [0.8 * b + 0.3 * d for b, d in zip(body, dust)]


def pickup():
    """A short rising blip."""
    def voice(t):
        f = 520 + 900 * t / 0.09
        return math.sin(2 * math.pi * f * t) * env(t, 0.09, curve=2.0) * 0.5
    return render(0.09, voice)


def hit():
    """A sharp crack: bright noise and a quick metallic ping."""
    crack = noise(0x417)
    def voice(t):
        ping = math.sin(2 * math.pi * 840 * t) * env(t, 0.08, curve=6.0)
        return 0.6 * crack() * env(t, 0.05, curve=5.0) + 0.4 * ping
    return render(0.1, voice)


def hurt():
    """A low, buzzing jolt, like a circuit taking a knock."""
    def voice(t):
        saw = 2.0 * ((t * 70) % 1.0) - 1.0
        wobble = 1.0 + 0.4 * math.sin(2 * math.pi * 30 * t)
        return saw * wobble * env(t, 0.22, curve=2.5) * 0.45
    return lowpass(render(0.22, voice), 0.25)


def teleport():
    """A falling sweep with a shimmer on top."""
    def voice(t):
        f = 1400 * math.exp(-t * 6.0) + 120
        phase = 2 * math.pi * (1400 / -6.0 * (math.exp(-t * 6.0) - 1) + 120 * t)
        shimmer = math.sin(2 * math.pi * 3 * f * t) * 0.2
        return (math.sin(phase) + shimmer) * env(t, 0.45, attack=0.02, curve=1.5) * 0.45
    return render(0.45, voice)


def craft():
    """Two short, rising chime notes."""
    def note(f, start, t):
        local = t - start
        if local < 0:
            return 0.0
        tone = math.sin(2 * math.pi * f * local) + 0.3 * math.sin(2 * math.pi * 2 * f * local)
        return tone * env(local, 0.18, curve=3.0)
    return render(0.3, lambda t: 0.35 * (note(660, 0.0, t) + note(990, 0.09, t)))


def shot():
    """A dry bang: a hard noise burst with a falling thump, then a short rattle."""
    blast = noise(0x5407)
    def voice(t):
        bang = blast() * env(t, 0.09, attack=0.001, curve=4.0)
        thump = math.sin(2 * math.pi * (160 - 500 * t) * t) * env(t, 0.1, attack=0.001, curve=5.0)
        return 0.7 * bang + 0.6 * thump
    return lowpass(render(0.18, voice), 0.5)


SOUNDS = {
    "mine": mine,
    "place": place,
    "pickup": pickup,
    "hit": hit,
    "hurt": hurt,
    "teleport": teleport,
    "craft": craft,
    "shot": shot,
}


def write(name, samples):
    # Normalize every sound to the same peak, 80% of full scale
    peak = max(1e-6, max(abs(s) for s in samples))
    frames = b"".join(struct.pack("<h", int(s / peak * 0.8 * 32767)) for s in samples)
    path = os.path.join(OUT, f"{name}.wav")
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(frames)
    print(f"wrote {os.path.normpath(path)} ({len(samples) / RATE:.2f} s)")


if __name__ == "__main__":
    for name, make in SOUNDS.items():
        write(name, make())
