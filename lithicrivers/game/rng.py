from lithicrivers.game.interfaces import Cloneable, ShutDownable

import msgspec


class SimpleRNG(Cloneable, ShutDownable, msgspec.Struct, frozen=False):
    state: int = None

    @classmethod
    def create(cls, seed: int):
        instance = cls(state=seed)
        return instance

    def choices(self, seq, weights=None, k=1):
        raise NotImplemented("todo lazy")

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