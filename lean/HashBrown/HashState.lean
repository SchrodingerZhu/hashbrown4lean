namespace HashBrown
namespace HashState

class HashState (η : Type) (α : Type) where
  finish : η → UInt64
  update : η → α → η

structure FxMixer where 
  state : UInt64

attribute [always_inline, inline] FxMixer.state

@[always_inline, inline]
def rotateLeft (x : UInt64) (k : UInt64) : UInt64 :=
  (x <<< k) ||| (x >>> (64 - k))

@[always_inline, inline]
def FxMixer.update [Hashable α] (s : FxMixer) (a : α) : FxMixer :=
  let h := hash a
  let rotated := rotateLeft h 5
  FxMixer.mk <| (s.state ^^^ rotated) * 0x517cc1b727220a95

instance [Hashable α] : HashState FxMixer α where
  finish s := s.state
  update s a := s.update a

instance : Inhabited FxMixer where
  default := FxMixer.mk 0

