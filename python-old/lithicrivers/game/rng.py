from lithicrivers.game.interfaces import Cloneable, ShutDownable

import msgspec


class SimpleRNG(Cloneable, ShutDownable, msgspec.Struct, frozen=False):
    state: int = None

    @classmethod
    def create(cls, seed: int):
        instance = cls(state=seed)
        return instance

    def shuffle(self, seq):
        """In-place shuffle using Fisher-Yates algorithm."""
        n = len(seq)
        for i in range(n-1, 0, -1):
            j = self.randint(0, i)
            seq[i], seq[j] = seq[j], seq[i]

    def choices(self, seq, weights=None, k=1):
        """Return a k-length list of elements chosen from seq, with optional weights."""
        if weights is None:
            return [self.choice(seq) for _ in range(k)]
        # Normalize weights
        total = sum(weights)
        cum_weights = []
        cumsum = 0
        for w in weights:
            cumsum += w
            cum_weights.append(cumsum)
        result = []
        for _ in range(k):
            x = self.random() * total
            # Find the first cum_weight > x
            for i, cw in enumerate(cum_weights):
                if x < cw:
                    result.append(seq[i])
                    break
        return result

    def choice(self, seq):
        return seq[self.randint(0, len(seq) - 1)]

    def randint(self, a: int, b: int) -> int:
        # LCG parameters (example values)
        self.state = (1664525 * self.state + 1013904223) % (2**32)
        return a + (self.state % (b - a + 1))

    def random(self) -> float:
        self.state = (1664525 * self.state + 1013904223) % (2**32)
        return self.state / 2**32

    def getstate(self):
        return self.state

    def setstate(self, state):
        self.state = state