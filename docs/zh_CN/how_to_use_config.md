# 关于配置文件

### 初始化配置文件

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
```source```文件夹下的```ini.toml```文件（**默认路径不可更改**）：

* ```[initialize]```里面可以定义的资源文件的路径；
* ```[character]```中定义角色名标识；

### 语音配置文件

```
voice
--rir
  length.toml
```

#### length.toml

对于每个角色，都要为其配置**语音文件夹**以及**语音配置文件**。
```
#length.toml

cast = [
    {name = "fem_rir_50520", length = 11},
    {name = "fem_rir_50521", length = 11},
    {name = "fem_rir_50522", length = 9},
]
```
在```length.toml```中，定义语音文件名，以及其长度用于**自动播放**。

### CG鉴赏配置文件

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
在```length.toml```中，定义CG文件名，序号以及**长度**，主CG的长度为**所有差分**的张数，差分CG的长度为1，差分CG的序号需要**严格保持**在主CG**之后**。


### 立绘配置文件

```
figure
--rir
  --z1
  --z2
  rir.toml
```
对于每个角色，都要为其配置**立绘文件夹**以及**立绘配置文件**。

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
在```rir.toml```中:

* ```[body]```定义立绘身体文件名，以及其长宽比。
* ```[face]```定义立绘表情文件名，以及其相对于身体图片的位移。
* ```[offset]```定义其立绘的相对偏移，较高的角色调正小数数值，较矮的角色调负小数数值，0可以当作水平线。

### 用户配置文件

```
savedata
1.toml
2.toml
...
extra.toml
user.toml
```
在```savedata```文件夹中，储存着与用户相关的配置。

#### 1.toml

```
#1.toml

script = "ky01"
block_index = 1
explain = "壬戌之秋，七..."
image_path = "./source/background/bg022a.png"
```

数字的```.toml```文件存储着存档相关的信息：

* ```script```指的是脚本名；
* ```block_index```指的是剧情快的块号；
* ```explain```指的是文本的描述；
* ```image_path```指的是背景图片存储的位置；

#### extra.toml

```
#extra.toml

[cg]
cg = 1022
```

```extra.toml```中，存储的是用户的extra信息，这些信息会随着用户在**游戏进程**中的解锁而修改：

* ```cg```是解锁CG的情况，使用位图索引进行储存，保持第一位为0；

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

```user.toml```中，存储的是用户的设置信息，这些信息会随着用户在**游戏设置**中的修改而修改：

* ```auto```可以设定自动等待的时长（单位为**秒**），以及自动播放是否等待语音结束；
* ```volume```是音量大小，可以调节各种音量大小；
