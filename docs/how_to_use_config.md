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

```initialize```里面可以定义的资源文件的路径；

```character```中定义角色名标识；

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

### 立绘配置文件

```
figure
--rir
  --z1
  --z2
  body.toml
  face.toml
```
对于每个角色，都要为其配置**立绘文件夹**以及**立绘配置文件**。

#### body.toml
```
#body.toml

cast = [
    {name = "rir_z1a0200", rate = 0.363},
    {name = "rir_z1b0200", rate = 0.389},
    {name = "rir_z1b0210", rate = 0.389},
]
```
在```body.toml```中，定义立绘身体文件名，以及其长宽比。

#### face.toml
```
#face.toml

cast = [
    {name = "a0050", x = 0.343, y = 0.0507},
    {name = "a0043", x = 0.337, y = 0.0808},
    {name = "a0049", x = 0.339, y = 0.0838},
    {name = "b0058", x = 0.332, y = 0.0728},
    {name = "b0059", x = 0.327, y = 0.0757},
    {name = "b0050", x = 0.337, y = 0.0530},
]
```
在```face.toml```中，定义立绘表情文件名，以及其相对于身体图片的位移。

### 用户配置文件

```
savedata
1.toml
2.toml
...
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

```script```指的是脚本名；

```block_index```指的是剧情快的块号；

```explain```指的是文本的描述；

```image_path```指的是背景图片存储的位置；

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

```auto```可以设定自动等待的时长（单位为**秒**），以及自动播放是否等待语音结束；

```volume```是音量大小，可以调节各种音量大小；
