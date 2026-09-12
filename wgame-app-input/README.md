# wgame-app-input

Independent buffered input streams. Overflow drops the oldest event. Termination wakes consumers, drains buffered events, and then ends the stream. Redraw events are handled separately by the window loop. See [input and timing](../docs/GUIDE.md#input-and-timing).
