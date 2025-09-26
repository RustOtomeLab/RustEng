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
