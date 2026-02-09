# About Configuration Files

> [!IMPORTANT]
> This document has been translated from Chinese to English by the DeepSeek large language model.

### Initialization Configuration File

```
source
ini.toml
```

#### ini.toml

```
#ini.toml

[initialize]
script_path = "./source/script/"
background_path = "./source/background/"
voice_path = "./source/voice/"
bgm_path = "./source/bgm/"
figure_path = "./source/figure/"
save_path = "./savedata/"

[character]
list = ["rir"]
```
The ```ini.toml``` file in the ```source``` folder **(default path cannot be changed)**:

* Resource file paths can be defined under ```[initialize]```
* Character identifiers are defined under ```[character]```

### Voice Configuration File

```
voice
--rir
  --length.toml
```

#### length.toml

For each character, you need to configure a **voice folder** and a **voice configuration file**.
```
#length.toml

cast = [
    {name = "fem_rir_50520", length = 11},
    {name = "fem_rir_50521", length = 11},
    {name = "fem_rir_50522", length = 9},
]
```
In ```length.toml```, define voice file names and their durations for **auto-play**.

### CG appreciation configuration file

```
cg
length.toml
```

#### length.toml

```
#length.toml

cast = [
    {name = "ev_rir_03_01", index = 1, length = 3},
    {name = "ev_rir_03_02", index = 2, length = 1},
    {name = "ev_rir_03_03", index = 3, length = 1},
    {name = "ev_rir_04_01", index = 4, length = 5},
    {name = "ev_rir_04_02", index = 5, length = 1},
    {name = "ev_rir_04_03", index = 6, length = 1},
    {name = "ev_rir_04_04", index = 7, length = 1},
    {name = "ev_rir_04_05", index = 8, length = 1},
    {name = "ev_rir_06_01", index = 9, length = 1},
]
```
In ```length.toml```, define the CG file name, sequence number, and **length**. The length of the main CG is the number of sheets of **all differences**, and the length of the difference CG is 1. The sequence number of the difference CG needs to be **strictly maintained** **after** the main CG.


### Character Sprite Configuration File

```
figure
--rir
  --z1
  --z2
  rir.toml
```
For each character, you need to configure a **sprite folder** and a **sprite configuration file**.

#### rir.toml
```
#rir.toml

[body]
cast = [
    {name = "rir_z1a0200", rate = 0.363},
    {name = "rir_z1b0200", rate = 0.389},
    {name = "rir_z1b0210", rate = 0.389},
    {name = "rir_noa0200", rate = 0.363},
    {name = "rir_nob0200", rate = 0.386},
]

[face]
cast = [
    {name = "z1a0050", x = 0.343, y = 0.0507},
    {name = "z1a0043", x = 0.337, y = 0.0808},
    {name = "z1a0049", x = 0.339, y = 0.0838},
    {name = "z1b0058", x = 0.332, y = 0.0728},
    {name = "z1b0059", x = 0.327, y = 0.0757},
]

[offset]
offset = 0.0
```
In ```rir.toml```:

* ```[body]``` defines character body sprite file names and their aspect ratios
* ```[face]``` defines facial expression file names and their offsets relative to the body image
* ```[offset]``` defines the relative vertical offset of the sprite (positive values for taller characters, negative for shorter characters, 0 as baseline)

### User Configuration Files

```
savedata
1.toml
2.toml
...
extra.toml
user.toml
```
The ```savedata``` folder stores user-related configurations.

#### 1.toml

```
#1.toml

script = "ky01"
block_index = 1
explain = "壬戌之秋，七..."
image_path = "./source/background/bg022a.png"
```

Numbered ```.toml``` files store save-related information:

* ```script``` refers to the script name
* ```block_index``` refers to the story block number
* ```explain``` refers to the text description
* ```image_path``` refers to the background image location

#### extra.toml

```
#extra.toml

[cg]
cg = 1022
```

In```extra.toml```, the user's extra information is stored, and this information will be modified as the user progresses **through the game**：

* ```cg```represents the unlocked CG condition, where bitmap indexing is used for storage, maintaining the first bit as 0；

#### user.toml

```
#user.toml

[auto]
delay = 5
is_wait = true

[volume]
main = 100.0
bgm = 100.0
voice = 100.0
```

```user.toml``` stores user settings information, which will be updated when users modify settings in the **game settings**:

* ```[auto]``` can set the auto-play delay (in **seconds**) and whether auto-play waits for voice to finish
* ```[volume]``` controls volume levels for different audio types
