# Bubblegum Spin: how the original plays
Source: https://www.lexaloffle.com/bbs/?pid=143764#p  (v1.1, Bee_Randon, no license)
Read-only reference. Never edit anything in reference/.

## Rules
- Match 3 or more gumballs of the same color to remove them from the mass.
- Gumballs left unattached after a pop also fall away.
- Clear all gumballs to start the next round.
- If the mass touches the edge: game over.
- Misses are defined by: having a gumball bounce against the walls 5 times without hitting anything, and having a gumball hit the pack but not clearing anything.
- Every 4 misses triggers a punishment. Each punishment is harsher than the
  last; from round 5 on, each new round punishes misses a bit more.
- Score is multiplied by the number of colors currently in play.

## Controls
- Left/right: aim. Up: reset the aimer to upright.
- X (or the fire button): fire. Hold Z: preview next gumballs + view score + view how many misses.
- Shots can bounce off the walls. You can see the next 2 gumballs in a popup when holding Z.

## Physics gotchas
- Shots make the whole mass spin. The force depends on how many gumballs
  are in the mass (fewer gumballs = a bigger spin from the same shot).
- Original PICO-8 uses fixed-point numbers; a port may behave slightly differently.

## Definition of done
1. Aim, fire and wall bounces feel like the original.
2. Matching, popping and unattached gumballs falling work.
3. The mass spins when hit, with force depending on the gumball count.
4. Rounds, misses/punishment, score multiplier and game over work.
5. Sound and music are present.
6. Windows and Linux builds can be downloaded from CI.
