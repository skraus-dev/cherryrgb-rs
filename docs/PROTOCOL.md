# Cherry Keyboard - USB HID protocol

This repository implements **two** different protocols:

1. **New protocol** (default `CherryKeyboard`): used by the newer boards
   (MX 3.0S, MX 8.0 TKL, G80-3000N, MX 10.0N, ...). 64-byte packets with a
   checksum, sent via `SET_REPORT` (report id 4) on the vendor interface,
   responses read from the interrupt IN endpoint. See `cherryrgb/src/models.rs`.
2. **Legacy protocol** (`cherryrgb::legacy`): used by older boards, so far
   only verified on the **MX BOARD 6.0 RGB (G80-3931, USB PID 0x00B8)**.
   Documented below - this protocol has not been publicly documented before.

## Legacy protocol (MX BOARD 6.0 RGB, PID 0x00B8)

### USB topology

The keyboard exposes three HID interfaces:

| Interface | Usage page / usage | Product string | Purpose |
| --------- | ------------------ | -------------- | ------- |
| MI_00     | 0x0001 / 0x0006    | Keyboard       | Standard boot keyboard (NKRO reports) |
| MI_01     | 0x000C / 0x0001    | Multimedia     | Consumer control |
| MI_02     | 0xFF01 / 0x0020    | Communication  | Vendor control channel (this protocol) |

The report descriptor of MI_02 defines three unnumbered reports:

- 64 byte **input** report (usage 0x02)   - interrupt IN endpoint (0x84)
- 64 byte **output** report (usage 0x01)  - interrupt OUT endpoint (0x03)
- 8 byte **feature** report (usage 0x03)  - `GET_REPORT` always answers `05 ...`

### Lighting frames

All lighting is done by the **host** streaming complete LED frames. The
firmware does **not** store frames - LEDs decay within a second when the
stream stops (the original utility streams at ~25 fps forever).

One frame = 9 output packets of 64 bytes each:

```
offset  content
0       report id placeholder (0x00, stripped by the OS)
1       0xc1                      magic
2       0x3d (chunk) / 0x21 (tail)
3       sequence number: 1..=8 for chunks, 9 for the tail packet
4..     key data: 15 keys * 4 bytes (chunks) or 7 keys * 4 bytes (tail),
        zero padded to 64 bytes
```

8 chunks * 15 keys + 7 tail keys = **127 key slots** per frame.

Per key 4 bytes:

```
[0] brightness   0x00..=0x3f (6 bit)
[1] blue         0x00..=0x3f
[2] green        0x00..=0x3f
[3] red          0x00..=0x3f
```

### Acknowledgements

Every output packet is answered by one input packet `c1 01 00 00 ...`.
Read (and discard) it before sending the next packet.

### Key events

Independent of the frame stream the keyboard emits input packets
`e0 23 c2 21 01 <state> 00 <HID keycodes ...>` when keys are pressed or
released. The original utility uses these to highlight physical keys in its
remapping editor. They are not required for lighting control.

### Physical key mapping

The mapping between the 127 key slots and the physical keys has not been
mapped yet. Slot indices are currently arbitrary; `static` (solid color)
works regardless. (Idea for future work: light one slot at a time and use the
`e0 23` key events to learn the mapping automatically.)

### Example frame (all 127 keys = full brightness red)

```
c1 3d 01 3f 00 00 3f 3f 00 00 3f 3f 00 00 3f ... 00
c1 3d 02 3f 00 00 3f 3f 00 00 3f ...              00
... (seq 03..08)
c1 21 09 3f 00 00 3f ... 00 00 00                 00
```
