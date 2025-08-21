# 关于配置文件

### 初始化配置文件

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
在```source```文件夹下的```ini.toml```文件（**默认路径不可更改**），```initialize```里面可以定义的资源文件的路径。```character```中定义角色名标识。

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
```
#body.toml

cast = [
    {name = "rir_z1a0200", rate = 0.363},
    {name = "rir_z1b0200", rate = 0.389},
    {name = "rir_z1b0210", rate = 0.389},
]
```
在```body.toml```中，定义立绘身体文件名，以及其长宽比。
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