# Scene Script

---

## Definition

**Scene script** files are text files with the ```.reg``` extension used to control scene changes. They manage the playback and switching of background images, subtitle text, voice-over audio, BGM audio, etc. They instruct the engine when to display which background, show which character, play which line of dialogue, and so on. Essentially, they are the core component governing the content.

---

## Documentation

> [!WARNING]
> To avoid potential encoding errors, we uniformly require that script files use **```UTF-8```** encoding!

A script consists of an initial **header block** and several **story blocks**. Blocks are separated by a blank line.

A line can start with ```#``` as a comment identifier; the engine will ignore such lines when reading the script.

---

### Header Block

The **Header Block** records relevant information about the script so the engine can read it and perform corresponding operations.

The **first line** specifies the protocol version, used to distinguish scripts following different version standards. The format is ```%version X```, where ```X``` is an integer representing the current script's protocol version.

---

### Story Block

```Story blocks``` record the storyline progression and current scene changes. Each block consists of several lines, with each line containing an ``operation`` that controls scene changes.

Here are the various operations and their usage:

* #### Background Image:

* * ```@bg bg1|0.1|0.2|1.5```: Here, ```@bg``` is the identifier for displaying a background image ( ```@cg``` can also be used; functionally identical, just for distinction in the script). Parts are separated by **vertical bars** ```|```.

* * ```bg1``` is the PNG image file used as the background. This file should be placed in the ```source/background``` folder under the root directory.

* * ```0.1|0.2|1.5```represent the x-offset, y-offset, and scale factor respectively, used to customize the background image's position and size. These values can **all be empty** (the ```|``` separators **can be omitted** if all are empty).

* #### BGM：

* * ```@bgm bgm2```: Here, ```@bgm``` is the identifier for playing background music. ```bgm2``` is the OGG audio file used as BGM. This file should be placed in the ```source/bgm``` folder under the root directory.

* #### Voice-over:

* * ```@voice rir|fem_rir_3```: Here, ```@voice``` is the identifier for playing dialogue voice-over. Parts are separated by **vertical bars** ```|```.

* * ```rir``` is the character's name.

* * ```fem_rir_3``` is the OGG audio file used for the voice-over. This file should be placed in the ```source/voice``` folder under the root directory.

* #### Subtitle:

* * ```Narrator“壬戌之秋，{nns}七月既望”```: This operation is split into two parts by **Chinese double quotation marks** ```“”```.

* * The first part is the speaker's name to be displayed.

* * The second part (inside the Chinese quotes) is the subtitle text. The speaker name can be omitted.

* * ```{nns}``` can be used to force a line break (up to **three lines** of text can be displayed).

* #### Character Sprite and Expression:

* * ```@fg rir|z1|rir_z1b0200|b0059|0|1000```: Here, ```@fg``` is the identifier for displaying a character sprite. Parts are separated by **vertical bars** ```|```.

* * ```rir``` is the character's name.

* * ```z1``` indicates the sprite's depth/layer (z-order).

* * ```rir_z1b0200``` is the PNG file for the sprite's body.

* * ```b0059``` is the PNG file for the sprite's facial expression.

* * ```0``` represents the sprite's horizontal position (from left to right: -2, -1, 0, 1, 2).

* * ```1000``` is a delay value (in milliseconds), which can **postpone** this operation. The delay can be **empty** (the ```|``` separator **can be omitted** if empty).

* * For detailed design and usage of sprites, please refer to the [Configuration File Documentation](how_to_use_config.md).

* #### Animation:

* * ```@move rar|z1|rar_z1a0200|z1a0041|-2|nod|3|4200```: Here, ```@move``` is the identifier for playing an animation. Parts are separated by **vertical bars** ```|```.

* * The initial parts must **match exactly** the corresponding sprite that the animation is applied to.

* * ```nod``` represents a nodding action. Other actions include ```tox``` (where x is the target position, e.g., ```to2``` to move the sprite to position 2).

* * ```3``` represents the number of times the action loops. ```-1``` means infinite looping.

* * ```4200``` is a delay value (in milliseconds), which can **postpone** this operation. The delay can be **empty** (the ```|``` separator **can be omitted** if empty).

* #### Label:

* * ```@label test```: Here, ```@label``` is the identifier for a label operation. The following text is the label name.

* * Label names **must be unique** within the **same** script. Labels can be used for choices and jumps.

* #### Jump:

* * ```@jump ky01:start```: Here, ```@jump``` is the identifier for a jump operation. The structure is divided by an **English colon** ```:```.

* * ```ky01``` is the target script filename.

* * ```start``` is the target label name within that script.

* * If the script filename is **empty** (but the **colon** must be present), it jumps to a label in the **current script**.

* * If the label name is **empty**, it jumps to the ```start``` label in the **target script**.

* #### Choice Branch:

* * ```
    @choose 3
    test
    Option1 :start
    Option2 ky02
    Option3 ky01:test
    ```

* * ```@choose``` is the identifier for the choice operation. The following number ```3``` indicates the number of choices in this branch.

* * ```test``` is the name of this choice branch. It can be customized and is used for the history log and save file display.

* * The subsequent lines list the choice options. The text at the beginning is the description of the option. The part after it specifies the jump logic, which follows the **same** rules as the ```jump``` operation.

