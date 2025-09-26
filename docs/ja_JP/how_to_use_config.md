# 設定ファイルについて

> [!IMPORTANT]
> この文書はテキスト大規模モデルDeepSeekによって中国語から日本語へ翻訳されました

### 初期化設定ファイル

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

```source```フォルダ内の```ini.toml```ファイル（**デフォルトパスは変更不可**）：

* ```[initialize]```ではリソースファイルのパスを定義できます
* ```[character]```ではキャラクター名の識別子を定義します

### 音声設定ファイル

```
voice
--rir
  length.toml
```

#### length.toml

各キャラクターに対して、**音声フォルダ**と**音声設定ファイル**を設定する必要があります。

```
#length.toml

cast = [
    {name = "fem_rir_50520", length = 11},
    {name = "fem_rir_50521", length = 11},
    {name = "fem_rir_50522", length = 9},
]
```

```length.toml```では、音声ファイル名とその長さを定義し、**自動再生**に使用します。

### 立ち絵設定ファイル

```
figure
--rir
--z1
--z2
  rir.toml
```

各キャラクターに対して、**立ち絵フォルダ**と**立ち絵設定ファイル**を設定する必要があります。

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


```rir.toml```では：

* ```[body]```は立ち絵の身体部分のファイル名とアスペクト比を定義します
* ```[face]```は立ち絵の表情ファイル名と身体画像に対する相対的な位置を定義します
* ```[offset]```は立ち絵の相対的なオフセットを定義します（背の高いキャラクターは正の小数値、背の低いキャラクターは負の小数値に調整し、0を水平線として扱います）

### ユーザー設定ファイル

```
savedata
1.toml
2.toml
...
user.toml
```

```savedata```フォルダには、ユーザー関連の設定が保存されます。

#### 1.toml

```
#1.toml

script = "ky01"
block_index = 1
explain = "壬戌之秋，七..."
image_path = "./source/background/bg022a.png"
```


数字の```.toml```ファイルはセーブデータ関連の情報を保存します：

* ```script```はスクリプト名を指します
* ```block_index```はストーリーのブロック番号を指します
* ```explain```はテキストの説明を指します
* ```image_path```は背景画像の保存場所を指します

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


```user.toml```にはユーザーの設定情報が保存され、ユーザーが**ゲーム設定**で変更すると更新されます：

* ```auto```では自動待機時間（単位は**秒**）と、自動再生が音声終了を待つかどうかを設定できます
* ```volume```は音量サイズで、各種音量を調整できます