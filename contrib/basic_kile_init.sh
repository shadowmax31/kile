# Sample kile config to achieve a basic layout similar to rivertile

# Ratio of display used by main area
riverctl map -repeat normal Super H send-layout-cmd kile "mod_main_ratio +0.01"
riverctl map -repeat normal Super L send-layout-cmd kile "mod_main_ratio -0.01"

# Number of views/windows/clients in the main area
riverctl map normal Super+Shift H send-layout-cmd kile "mod_main_amount +1"
riverctl map normal Super+Shift L send-layout-cmd kile "mod_main_amount -1"

# arg1, Tags to apply command to: default|focused|all|0..32
# arg2, Name to assign to this layout
# arg3, Layout definition
riverctl map normal Super+Control Up    send-layout-cmd kile "focused U ((h: v v) 1 0.65 0)"
riverctl map normal Super+Control Down  send-layout-cmd kile "focused D ((h: v v) 1 0.65 1)"
riverctl map normal Super+Control Left  send-layout-cmd kile "focused L ((v: h h) 1 0.65 0)"
riverctl map normal Super+Control Right send-layout-cmd kile "focused R ((v: h h) 1 0.65 1)"
riverctl map normal Super+Control D     send-layout-cmd kile "focused Deck deck"
riverctl map normal Super+Control F     send-layout-cmd kile "focused Full full"

# Tell river to use kile as its layout generator
riverctl default-layout kile

# Note: nothing after this line will be run
exec kile --namespace kile --layout "((v: h h) 1 0.65 1)"
