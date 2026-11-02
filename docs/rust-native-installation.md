# Rust ile sade Vexen kurulumu: Ponytail yaklaşımının uyarlanması

Tarih: 2 Kasım 2026. Durum: GitHub kaynak araştırması ve uygulanacak kurulum taslağı. Bu belgede önerilen `setup` akışı ve plugin paketleri henüz uygulanmış değildir. Kaynak kod, ürün kurulumu ve release akışı bu çalışma sırasında değiştirilmedi.

## Karar ve amaç

**Kurulumun sorumluluğunu Vexen üstlenecek; çalışma zamanı ve kurulumun iş mantığı Rust olacak.** Kullanıcı binary, MCP ayarı, skill ve proje başlangıcını elle birleştirmeyecek. Ponytail'den alınacak yaklaşım, agent'a uygun giriş noktası ve bütünleşik etkinleştirmedir.

Ürün Rust olarak kalır. Plugin manifestleri JSON, davranış yönergeleri Markdown olabilir; bunlar ayrı bir uygulama çalışma zamanı oluşturmaz. İlk native binary'yi indirebilmek için gereken küçük Shell/PowerShell veya işletim sistemi paketleme adımı, platform bootstrap katmanıdır. Kullanıcıdan Node, npm, Python veya Rust derleyicisi kurmasını isteyen birincil kullanım yolu hedeflenmez. Kullanıcının seçtiği agent'ın kendi gereksinimleri Vexen'in bağımlılıklarıyla karıştırılmamalı.

Mevcut npm/npx yolu geriye uyumluluk için isteğe bağlı kalabilir. Yeni native yol hazır ve doğrulanmış olmadan çalışan kurulum seçenekleri kaldırılmamalı. Geliştirme/deney araçlarını sırf dil birliği için yeniden yazmak bu işin kapsamı değildir; yeni ürün çalışma zamanı ve setup mantığı Rust'a ait olmalı.

Oxocarbon renkleri, Geist/Geist Mono ve mevcut logo sabittir.

## Mevcut kopukluklar

İncelenen dosyalar: `Cargo.toml`, `src/cli/install.rs`, `src/install/{config,registry}.rs`, `src/cli/doctor.rs`, `skills/vexen/SKILL.md`, `npm/README.md`, `docs/getting-started.md`.

- Rust çekirdeği ve agent ayarlarını düzenleyen Rust kodu zaten var. İkinci bir genel MCP yöneticisi yazmaya gerek yok.
- `init`, MCP bağlantısı ve skill yerleşimi ayrı kullanıcı adımları.
- npx başlatıcısı binary'yi önbelleğe alıyor, fakat skill doğrudan PATH üzerindeki `vexen` komutunu bekliyor.
- npx üzerinden her başlatma hâlâ Node gerektirir; binary'nin önbelleğe alınması npx komutunu bağımsız Rust çalıştırmasına dönüştürmez.
- `doctor`, mevcut hâliyle proje/depo sağlığını kontrol ediyor; agent'ın gerçekten bağlanmasını veya skill'i yüklemesini kanıtlamıyor.
- Desteklenen agent bulunmadığında `install` bilgi mesajıyla başarılı çıkabiliyor. Birleşik setup bunu “kullanıma hazır” diye göstermemeli.

## GitHub araştırması: hangi parça Rust ile yapılmış?

### 1. En yakın kurulum örneği: jujutsu-mcp

Kaynaklar: [depo](https://github.com/LLawli/jujutsu-mcp), [Rust setup kodu](https://github.com/LLawli/jujutsu-mcp/blob/main/src/setup.rs), [Cargo manifesti](https://github.com/LLawli/jujutsu-mcp/blob/main/Cargo.toml).

`jujutsu-mcp setup`, çalıştırılabilir dosyanın konumunu belirliyor, gerektiğinde kalıcı bir yere kopyalıyor ve kurulu agent'ların CLI'ları üzerinden MCP kaydı yapıyor. Dry-run ve agent seçimi mevcut. Kod Rust; `rmcp`, `toml_edit`, `std::process::Command` kullanımı Vexen'in mevcut yapısıyla yakın.

**Alınacak desen:** önce plan oluştur, sonra kurulumu yap; MCP komutuna geçici build/download yolu değil, kalıcı native binary yolu yaz. Agent'ın resmî yönetim komutunu kullanmak mümkünse ayar dosyası biçimini yeniden uygulamaktan kaçın.

**Olgunluk sınırı:** GitHub kaydı 2 Ekim 2026 başlangıçlı; incelenen paket `0.1.4`. Bu, yaklaşık bir aylık bir uygulama örneği; bağımlılık olarak uzun süreli güvenilirliği kanıtlanmış değil. Kaynak incelendi, kurulum tarafımızdan çalıştırılmadı. Vexen'e topluca kopyalanması veya bağımlılık yapılması önerilmiyor.

### 2. Olgun Rust dağıtım aracı: cargo-dist / dist

Kaynaklar: [depo](https://github.com/axodotdev/cargo-dist), [Rust workspace manifesti](https://github.com/axodotdev/cargo-dist/blob/main/Cargo.toml), [dağıtım belgesi](https://github.com/axodotdev/cargo-dist/blob/main/README.md).

2022 başlangıçlı Rust projesi; incelenen workspace sürümü `0.34.0`. Native binary, platform arşivi, kurucu ve release manifesti üretimi için uygun aday. Node üzerinden çalışan bir Vexen servis katmanı gerektirmez.

**Kullanım sınırı:** dağıtım aracıdır; Vexen'in MCP kaydını, skill yerleşimini veya proje belleğini kendiliğinden birleştirmez. Vexen'in mevcut release iş akışıyla maliyet karşılaştırması yapılmalı; sadece tek komut görüntüsü elde etmek için tüm pipeline değiştirilmemeli. Üretilen kurucu dosyaların Shell/PowerShell olması, Vexen'in çalışma zamanını JavaScript'e taşımaz.

### 3. Rust geliştiricileri için ek kanal: cargo-binstall

Kaynaklar: [depo ve kurulum davranışı](https://github.com/cargo-bins/cargo-binstall), [Rust workspace](https://github.com/cargo-bins/cargo-binstall/blob/main/Cargo.toml).

2020 başlangıçlı, Rust ile yazılmış binary kurulum aracı. Hazır release dosyalarını kullanabiliyor; başarısız çözümlemede başka kaynaklara veya kaynak derlemeye geri dönüş yolları var.

**Uygun rol:** zaten bu aracı kullanan geliştiriciler için isteğe bağlı dağıtım kanalı. Yeni başlayan kullanıcıya önce başka bir kurucu kurdurmak ana yol olmamalı. Vexen'in crate adı, yayımlanmış paket kimliği ve release eşleşmeleri doğrulanmadan `cargo binstall vexen` çalışıyor diye belgelenmemeli. Kaynak/derleme geri dönüşü ve doğrulama politikası ayrıca incelenmeli.

### 4. Rust ile otomatik yapılandırma mümkün: Roblox örneği

Kaynak: [Roblox/studio-rust-mcp-server](https://github.com/Roblox/studio-rust-mcp-server).

README, native programın istemci ayarlarını ve Studio plugin kurulumunu üstlendiği bir akış gösteriyor. Ancak depo arşivlenmiş; bakımcı aktif geliştirmenin Studio'nun yerleşik MCP sunucusuna geçtiğini açıklıyor.

**Karar:** “indir, çalıştır, istemciyi yeniden başlat” kullanıcı deneyimi için tarihsel referans. Yeni dependency veya bakım temeli olarak seçilmemeli. Vexen'in mimarisi Roblox'un web sunucusu/Studio köprüsünü gerektirmiyor.

### 5. İncelenip seçilmeyenler

- [Ingen](https://github.com/hethon/ingen): Rust hedef adları ve cargo-dist şablonları kullanıyor, fakat incelenen kurulum yolu `npm install --global ingen-cli`; depo dili Shell. Bunu “Rust ile aynı işi yapan araç” diye sınıflandırmak yanlış olur. Rust odaklı ana yol için seçilmedi.
- [Brightwing MCP Manager](https://github.com/Brightwing-Systems-LLC/mcp-manager): Rust ağırlıklı fakat masaüstü uygulaması, proxy ve daemon içeren daha geniş bir yönetim katmanı. Vexen'in ihtiyacı tek ürün setup'ı; bu platformu taşımak kapsamı ve bağımlılıkları büyütür.
- [add-mcp](https://github.com/neon-solutions/add-mcp): istemci formatları için referans olabilir; TypeScript/Node uygulaması Vexen'in native kurulum motoru yapılmayacak.

**Sonuç:** Aynı kullanıcı deneyiminin parçalarını Rust ile uygulayan projeler var. Native dağıtımda olgun seçenekler mevcut; agent kaydı ve skill etkinleştirmeyi birlikte yapan, Vexen'e doğrudan takılacak tek bir olgun kütüphane bu incelemede doğrulanmadı. En küçük çözüm, mevcut Vexen Rust kodunu birleştirmek.

## Hedef kullanıcı akışı

### Native ana yol

1. Kullanıcı agent'ını ve işletim sistemini seçer.
2. Resmî release'in doğrulanan native binary'si kalıcı kullanıcı konumuna yerleştirilir. Rust derleyicisi, Node veya Python gerekmez.
3. Aynı binary içindeki setup, seçili agent'a MCP bağlantısını ve kullanım yönergesini kurar.
4. Kullanıcı mevcut Git projesinde belleği bir kez başlatır. Proje başlangıcı aynı kurulum oturumunda açıkça seçilmişse setup bunu da yapabilir.
5. Sonuç, tamamlanan ve eksik adımları ayrı gösterir; istemci yeniden başlatması veya güven onayı gerekiyorsa açıkça söyler.

Önerilen arayüz — **henüz mevcut komut değildir**:

```text
vexen setup
```

Bu, yeni bir yönetim platformu değil; mevcut init/install kontrollerini ve skill yerleşimini koordine eden küçük bir Rust komutudur. Mevcut alt komutlar ileri kullanım ve otomasyon için korunabilir. Daha önce desteklenen güvenli davranışlar setup uğruna atlanmamalı.

### Plugin yolu

Ponytail gibi agent'ın kendi marketplace/plugin arayüzü kullanılabilir. Paket; manifest, skill/komut yönergeleri ve native MCP bağlantısını bir araya getirir. Rust kodu plugin setup ve gerekirse yaşam döngüsü hook'larını yürütebilir; Node hook zorunluluğu eklenmemeli.

**Bootstrap gerçeği:** bir manifest, olmayan native binary'yi kendiliğinden çalıştıramaz. Sadece marketplace manifesti ekleyerek “iki komutla tamamen hazır” denemez. İlgili plugin dağıtımının platform binary'sini güvenilir biçimde teslim etmesi veya önceden kurulu native Vexen'e açıkça dayanması gerekir. Bu davranış her agent'ın gerçek plugin kurulumunda doğrulanmalı. Plugin kurulumunda otomatik native indirme/çalıştırma desteği varsayılmamalı.

Native teslim henüz çözülmemişse dürüst ilk akış: native kurulum + isteğe bağlı agent plugin'i. Marketplace komutları ancak yayımlanan paket ve testler hazır olduğunda README'ye çalışan talimat olarak eklenmeli.

### skills.sh yolu

Skill dağıtımı isteğe bağlı giriş noktası olabilir; skill kurulması native Vexen'in veya proje belleğinin kurulması demek değildir. Skill öncelikle MCP araçlarını kullanabilir; CLI yolu gerekiyorsa kurulan binary'nin gerçek konumunu çözmelidir. Kullanıcıya ayrıca PATH düzeltme ödevi bırakılmamalı. `skills.sh` ve npx ana native yolun önkoşulu olmayacak.

## Rust uygulamasının sınırları

### Mevcut parçaları birleştir

`clap`, `serde_json`, `toml_edit`, `rmcp`, `sha2` ve mevcut `src/install` yapısı kullanılmalı. Önce mevcut modüller arasında küçük bir setup koordinasyonu dene; yeni registry, dependency injection çatısı, ikinci installer motoru veya genel plugin yöneticisi kurma.

Skill metni mevcut dosyadan binary'ye gömülebilir veya aynı sürümlü release içinden dağıtılabilir. Manifest ve skill sürümlerinin binary ile eşleşmesi test edilmeli. Aynı metnin bağımsız kopyalarının zamanla ayrışmasına izin verilmemeli.

### Kalıcı executable ve güvenli istemci kaydı

MCP komutunda Node/npx yerine native Vexen'in doğrulanmış yolu kullanılmalı. Programın kurulduğu konum boşluk, Unicode veya Windows yol ayracı içerdiğinde de çalışmalı; komut/argümanlar shell metni birleştirmesiyle oluşturulmamalı.

Agent'ın desteklenen kendi CLI'ıyla kayıt yapılabiliyorsa önce bunun sürüm/kapsam/çıkış kodu davranışını doğrula. Yoksa mevcut güvenli ayar adaptörlerini kullan. Kendi CLI'ını kullanmak otomatik olarak izinleri ve dosyaları koruduğunu kanıtlamaz; her yol ayrı test ister.

### Global araç, proje belleği

Binary ve agent entegrasyonu kullanıcı kapsamında bir kez kurulabilir. `.vexen/` belleği her proje için ayrı kalır. Bir agent'ın bulunması, bütün projelere otomatik bellek açma izni sayılmaz. Setup yalnızca seçilen projeyi başlatsın; rastlanan tüm Git depolarını dolaşmasın. Tekrar kurulum eski kayıtları silmesin.

### Doğrulama katmanlarını ayır

- **Native çalıştırma:** binary sürümü ve desteklenen platform.
- **Proje:** doğru Git kökü, başlatılmış ve sağlıklı veri deposu.
- **Yapılandırma:** seçilen agent'ta doğru MCP girdisi ve skill/komut dosyası.
- **Protokol:** gerçek native süreçle araç keşfi ve bir kontrol çağrısı.
- **Agent:** mümkünse gerçek istemcinin bu kaydı yüklemesi ve çağırması; değilse “henüz doğrulanmadı”.

Doğrudan bir test istemcisinin Vexen'e bağlanması, kullanıcının agent'ının yeni ayarı yüklediğini kanıtlamaz. `doctor` çıktısı bu farkı gizlememeli. İstemci bulunmaması, manuel adım veya başarısız skill kurulumu “her şey hazır” sonucuna dönüşmemeli.

### İşlem güvenliği

Mevcut backup, atomik yazma, kayıp güncelleme kontrolü, symlink reddi, idempotency ve sahip olunan girdiyi kaldırma kuralları korunmalı. Setup yalnızca kendi oluşturduğu adımları geri alabilir; var olan belleği veya yabancı plugin/MCP ayarlarını silmemeli. Kısmi başarısızlık hangi adımda kaldığını ve tekrar denenebilir yolu göstermeli.

Plugin ve eski global MCP kurulumu birlikte bulunduğunda çift sunucuyu tespit et; kullanıcıya açık bir geçiş yolu göster. Sessizce yabancı kayıtları kaldırma veya araç onay kurallarını gevşetme. Kurulum onayı sınırsız test çalıştırma, sohbet kaydetme veya arka plan hizmeti ekleme izni değildir.

## Uygulama sırası

1. Native binary yolu ile mevcut skill davranışını uyumlu hâle getir; kullanım yönergesi MCP ve native CLI yollarını açık ayırsın.
2. Mevcut Rust init/install parçalarını tek setup akışında koordine et; sürümlü skill dağıtımı ekle.
3. Native kurucuyu mevcut release sözleşmesini koruyarak üret; cargo-dist geçişi ancak bakım faydası doğrulanırsa yapılsın.
4. Kurulum/protokol/agent doğrulamasını ayrı durumlar olarak göster.
5. Bir agent için tam plugin paketini gerçek kurulumda doğrula; sonra diğer agent'lara genişlet.
6. README'de agent seçimi ve kısa native başlangıcı öne çıkar. Eski manuel/npx yollarını alternatif olarak açıkça etiketle.

## Bitti sayılma ölçütleri

- Windows, macOS ve Linux'ta desteklenen platformların temiz kurulumları test edilmiş; gerekli runtime olarak Node/Python/Cargo istenmiyor.
- Native dosya ve platform seçimi, bozuk/eksik indirme ve doğrulama hataları test edilmiş; başarısız indirme çalışan binary'yi bozmaz.
- Agent'ı kurduktan sonra kullanıcının ikinci kez MCP JSON'u veya PATH'i elle düzeltmesi gerekmiyor.
- Aynı setup ikinci kez çalışınca duplicate server/skill oluşmuyor.
- Proje belleği başlangıç kapsamı açık; güncelleme ve kaldırma kullanıcı kayıtlarını koruyor.
- Skill'in yönergeleri gerçekten çalışıyor; yalnızca dosyanın diske yazılması yeterli sayılmıyor.
- Gerçek istemci ve native Vexen ile en az bir kayıt kontrolü doğrulanmış; yeniden başlatma/güven gereklilikleri belgelenmiş.
- İnternet kapalı kullanım ayrıca denenmiş; npm önbelleği bulunması offline test yerine geçmiyor.
- İstemci yok, kapsam desteklenmiyor veya kurulum kısmi kaldı durumlarında yanlış başarı mesajı yok.
- Native/plugin/skill yolları aynı bellek motoruna gidiyor; izinler ve veri kapsamı değişmiyor.
- Ana kurulum yolu kısa; ileri seçenekler ve teknik kavramlar ilk denemeyi engellemiyor.

## Doğrulama durumu ve kapsam dışı işler

Bu oturumda GitHub README'leri, Cargo manifestleri, jujutsu-mcp'nin setup kaynak kodu ve depoların güncel arşiv/bakım durumları incelendi. Üçüncü taraf kurucular çalıştırılmadı; Vexen'e yeni dependency eklenmedi. Özellikle genç örneklerin kaynak desenleri, üretim güvenilirliği iddiası değildir.

Mevcut Rust kurucusu yeniden yazılmayacak; sadece eksik koordinasyon tamamlanacak. Genel plugin marketplace'i, GUI, daemon, otomatik sohbet kaydı, zorunlu model/embedding ve yeni ağ servisi bu işin kapsamı değildir. Kurulum kolaylığı ile gerçek görev başarısı ayrı kabul alanlarıdır.

## Derin inceleme: kurulumun eksik kalan teknik sınırları

Bu ek incelemede README açıklamalarının yanında Rust kaynakları ve bağımlılık manifestleri kontrol edildi. Aşağıdaki araçlar Vexen'e kurulmadı; yetenekleri entegrasyon başarısı olarak sunulmuyor. Commit bağlantıları araştırma anındaki sürümü sabitler; uygulamadan önce seçilecek yayımlanmış sürüm ayrıca doğrulanmalı.

### Kaynak kaydı

| Kaynak | İncelenen sabit kaynak | Kanıtladığı parça | Kullanım kararı |
|---|---|---|---|
| jujutsu-mcp | [`setup.rs`, e59e9e6](https://github.com/LLawli/jujutsu-mcp/blob/e59e9e62e580c7675241227fea98d50a5710d1c3/src/setup.rs) | Rust kurulum planı, kalıcı binary, agent CLI çağrısı ve Windows değiştirme yolu | Yakın uygulama örneği; genç proje, bağımlılık önerisi değil |
| cargo-binstall | [`atomic-file-install`, 8511124](https://github.com/cargo-bins/cargo-binstall/blob/851112458b5f28a29e5bc14a690c3f283d9dcc6f/crates/atomic-file-install/src/lib.rs) | Geçici dosya, izin aktarımı, aynı dosya sisteminde yerleştirme ve Windows değiştirme davranışı | Mevcut Rust yazma davranışını karşılaştırmak için referans |
| cargo-binstall | [`fs-lock`, 8511124](https://github.com/cargo-bins/cargo-binstall/blob/851112458b5f28a29e5bc14a690c3f283d9dcc6f/crates/fs-lock/src/lib.rs) | Paylaşımlı/özel dosya kilidi; bloklayan ve try-lock ayrımı | Eşzamanlı setup testi için referans; yeni kilit katmanını peşinen ekleme |
| self_update | [`README`, 1524279](https://github.com/jaemk/self_update/blob/15242797438a049ddd55d6e536391346dc006c91/README.md) | Rust ile release indirme, seçenekli doğrulama ve kurulum öncesi kontroller | Mevcut release akışı korunursa değerlendirilebilecek aday |
| self_update | [`verify_signature`, aynı commit](https://github.com/jaemk/self_update/blob/15242797438a049ddd55d6e536391346dc006c91/src/update.rs#L2269) | Boş anahtar listesiyle fonksiyon doğrulama yapmadan başarılı döner | Özelliğin adını kullanmak yeterli değil; fail-closed politika ayrıca gerekli |
| self-replace | [`README`, f74d7bd](https://github.com/mitsuhiko/self-replace/blob/f74d7bd1ab1c29b1bfe71a59f4bcae545ebdd34e/README.md) ve [uygulama/sınırlar](https://docs.rs/self-replace/1.5.1/self_replace/) | Çalışan executable'ın platforma uygun değiştirilmesi ve silinmesi | Yalnızca native dosya değiştirme gerçekten gerekirse dar yardımcı aday |
| axoupdater | [`README`, b4e5c73](https://github.com/axodotdev/axoupdater/blob/b4e5c73f26293eafd2410066ed61a86cbfede357/README.md) | cargo-dist kurulum makbuzuna dayalı Rust updater | cargo-dist seçilirse alternatif; diğer updater ile birlikte varsayılan ekleme |
| Rust standart kütüphanesi | [`include_str!`](https://doc.rust-lang.org/std/macro.include_str.html) | UTF-8 metni derleme anında binary'ye gömme | Az sayıdaki mevcut skill dosyası için önce standart çözümü değerlendir |

### Yeni Rust adaylarının olgunluğu ve sınırları

**self_update:** depo 2017 başlangıçlı; incelenen Cargo manifestinde `1.3.3`, Rust 1.88 alt sınırı ve MIT lisansı bulunuyor. Bu bir kaynak snapshot'ıdır; crates.io'da aynı sürümün yayımlandığı bu araştırmada ayrıca doğrulanmadı. Checksum ve imza kontrolleri seçenekli özelliklerdir. Arşiv ve çıkarılmış binary için ayrı doğrulama noktaları sunması yararlı; fakat paket seçmek güven politikasını kendiliğinden kurmaz. Kaynak: [manifest](https://github.com/jaemk/self_update/blob/15242797438a049ddd55d6e536391346dc006c91/Cargo.toml).

**self-replace:** 2023 başlangıçlı, daha dar kapsamlı Rust kütüphanesi. HTTP indirme, imza politikası, agent yapılandırması veya veri migrasyonu yapmaz. Özellikle Windows'ta çalışan dosyanın değiştirilmesini ele alır; güç kesintisi gibi durumlarda geçici dosya kalabilmesi belgelenmiştir. “Her aşama tam atomik ve iz bırakmaz” iddiası çıkarılmamalı.

**axoupdater:** 2024 başlangıçlı Rust projesi; incelenen manifest `0.10.3`. cargo-dist tarafından oluşturulan kurulum makbuzlarını kullanması mevcut dağıtım tercihine bağımlılık getirir. Vexen'in manuel/npx kurulumu bugün böyle bir makbuz üretmiyor. Geçiş maliyeti ve mevcut kullanıcılar için davranış belirlenmeden doğrudan devreye alınmamalı. Kaynak: [manifest](https://github.com/axodotdev/axoupdater/blob/b4e5c73f26293eafd2410066ed61a86cbfede357/Cargo.toml).

### İndirme, doğrulama ve yerleştirme aynı şey değil

Önerilen sıra: hedef/sürüm seçimi → geçici indirme → boyut ve içerik doğrulaması → varsa zorunlu imza/provenance doğrulaması → güvenli arşiv açma → staged binary sürüm kontrolü → kalıcı yerleştirme → agent bağlantısı → gerçek kontrol çağrısı.

Checksum dosyasının aynı release ile sunulması aktarım bütünlüğünü kontrol eder; tek başına bağımsız yayıncı kimliği doğrulaması değildir. İmza kullanılacaksa anahtar kaynağı, boş anahtar listesi, yanlış anahtar, eksik imza ve anahtar yenileme davranışları ayrı test gerektirir. `self_update` kaynak kodunda boş anahtar listesi başarılı sonuç verdiğinden Vexen, imza zorunlu olduğu modda bu girdiyi kendisi reddetmelidir. Aynı prensip desteklenmeyen platform veya eksik doğrulama verisinin sessizce atlanmaması için de geçerlidir.

Vexen'in mevcut release iş akışında build-provenance attestation adımı `continue-on-error: true` taşıyor; mevcut npm başlatıcısı SHA-256 kontrolü yapıyor. Dolayısıyla bugün bütün artifact'ların zorunlu imza/provenance kontrolünden geçtiği söylenemez. Native yol hangi doğrulamayı garanti ediyorsa onu açıkça belgelemeli; farklı güven düzeylerini tek “verified” etiketi altında birleştirmemeli.

Arşiv açmada hedef dışına çıkan yollar, symlink kaçışı, aşırı büyük içerik ve beklenmeyen executable reddedilmeli. Binary sürüm kontrolü doğrulanmamış dosyayı çalıştırmadan önce yapılacak güvenlik kontrollerinin yerine geçmez. Kaynak derlemeye veya üçüncü taraf binary kaynağına beklenmedik geri dönüş, ana kullanıcı akışında sessiz gerçekleşmemeli.

### Dosyayı değiştirmek bütün setup'ı atomik yapmaz

Bir binary'nin güvenli değiştirilmesi; skill, agent ayarı ve proje başlangıcının tek transaction olduğu anlamına gelmez. Küçük bir işlem planı, sahip olunan dosyaları ve tamamlanan adımları kaydetmeli. Bir adım başarısızsa mevcut çalışan kurulum ve proje belleği korunmalı; geri alma yalnızca Vexen'in değiştirdiği, arada kullanıcı tarafından değişmemiş girdilere uygulanmalı.

`cargo-binstall` kaynakları hem hedefin yakınında geçici dosya kullanımını hem platforma özel değiştirme/kilit davranışlarını gösteriyor. Bunlar Vexen'in mevcut atomik ayar yazımıyla karşılaştırılacak örneklerdir. Bütün workspace'i taşımak veya yeni bir installer framework'ü eklemek gerekmiyor. Kilit gerekiyorsa süresiz bekleme yerine zaman aşımı/yeniden deneme ve anlaşılır hata tasarlanmalı.

### Agent CLI kullanımı da denetlenmeli

İncelenen jujutsu-mcp `run_step`, argüman dizisiyle süreç başlatıyor ve stdin'i kapatıyor; fakat bu fonksiyonun kendisinde süreç zaman aşımı yok. Kapalı stdin, komutun her durumda hızlı biteceğini kanıtlamaz. Vexen agent kayıt komutu çalıştıracaksa süre sınırı, çıkış kodu, sınırlı stderr ve yarıda kalan işlem davranışını test etmeli. Varsa “eski kaydı kaldır, yenisini ekle” yaklaşımı ekleme hatasında çalışan bağlantıyı kaybettirmemeli.

Agent CLI'ı kullanmak mevcut güvenli dosya adaptörlerinden otomatik olarak üstün değildir. CLI sürüm sözleşmesi, kapsamı, hata davranışı ve ayar koruması doğrulanmıyorsa mevcut Rust adaptörü daha uygun olabilir. Karar her agent için ayrı alınmalı; Vexen'in tek entry sahipliği ve onay ayarlarını koruma şartı değişmez.

### Skill teslimi ve sürüm eşleşmesi

Mevcut az sayıdaki Markdown skill için `include_str!`, ek dependency olmadan tek native binary ile dağıtım sağlayabilir. Bu, derleme anında metin gömer; agent'a kurulum, aktivasyon ve güncelleme ayrıca uygulanır. Yol ve UTF-8 davranışı desteklenen build platformlarında kontrol edilmeli.

Binary, plugin ve skill için desteklenen sürüm eşleşmesi tanımlanmalı. Güncelleme eski agent ayarını ve kullanıcı değişikliklerini korumalı; kaldırma yalnızca sahip olunan skill/MCP dosyalarını kaldırmalı. Rust binary'yi yenilemek açık agent oturumunun skill'i yeniden yüklediği anlamına gelmez.

## Mimari seçenekler: birini seç, hepsini ekleme

| Yol | Ne zaman uygun? | Ek yük / sınır | Öneri |
|---|---|---|---|
| Mevcut release + Rust setup | İlk sade native kurulum | Mevcut kurucu sözleşmesi ve güvenlik adımları tamamlanır | İlk uygulanacak en küçük tam çözüm |
| cargo-dist + gerekiyorsa axoupdater | Dağıtım bakımını standartlaştırmak somut fayda sağlıyorsa | Release iş akışı ve kurulum makbuzu geçişi | Ölçülmüş bakım ihtiyacında değerlendir |
| Mevcut release + self_update | Açıkça istenen bir native update işlevi gerekiyorsa | HTTP/arşiv/TLS özellikleri, doğrulama politikası ve platform testleri | Daha sonraki ayrı iş; onboarding önkoşulu değil |
| Dar self-replace kullanımı | Güvenilir staged binary zaten eldeyse ve çalışan dosya değişimi gerekiyorsa | İndirme ve doğrulama hâlâ Vexen'e ait | Sadece bu alt problem için değerlendir |

Arka planda sürüm kontrolü veya MCP başlatılırken ağ erişimi eklenmemeli. Önce çalışır native setup; otomatik güncelleme motoru ancak ayrı, açık bir ihtiyaçla ele alınmalı. Paket yöneticisiyle kurulan dosyalar setup/update tarafından sessizce sahiplenilip değiştirilmemeli; ilgili paket yöneticisinin güncelleme yolu korunmalı.

## Doğrulama matrisi: geliştirici makinesinde çalışması yeterli değil

Aşağıdaki durumlar gelecekteki uygulamanın kabul testleridir; bu araştırmada çalıştırılmış sonuçlar değildir.

| Senaryo | Beklenen kanıt |
|---|---|
| Node/Python/Cargo bulunmayan temiz desteklenen işletim sistemi | Native Vexen kurulup MCP süreci başlar; kullanıcıdan ek runtime istenmez |
| Boşluk/Unicode içeren kullanıcı dizini | Skill ve MCP komutu gerçek istemcide doğru executable'ı açar |
| Eksik/bozuk indirme, yanlış platform, checksum uyuşmazlığı | Eski binary ve agent ayarları korunur; hazır mesajı verilmez |
| Zorunlu imza modunda anahtar/imza eksik veya yanlış | Açma/çalıştırma başlamadan işlem reddedilir |
| Windows'ta kullanımda olan binary | Platforma uygun değiştirme veya anlaşılır erteleme; mevcut çalışan dosya bozulmaz |
| Aynı anda iki setup | Tek tutarlı kurulum; duplicate skill/server veya kayıp kullanıcı ayarı yok |
| Binary başarılı, agent kaydı başarısız | Kısmi durum ve tekrar denenebilir adım görünür; proje belleği korunur |
| Agent kayıt CLI'ı takılır veya hata verir | Süre sınırı ve anlaşılır hata; süresiz bekleme ve yanlış başarı yok |
| Skill kullanıcı tarafından düzenlenmiş | Sessiz overwrite yok; sahiplik/çatışma açıkça gösterilir |
| Plugin ve eski global MCP birlikte mevcut | Çift kayıt tespit edilir; geçiş kullanıcı girdisini silmeden yapılır |
| İnternet kesilmiş, kalıcı native kurulum hazır | MCP okuma/kaydetme yerel çalışır; başlangıçta registry/update isteği yapılmaz |
| Yapılandırma yazılmış fakat agent yeniden başlamamış | “Yapılandırıldı” ile “istemcide doğrulandı” ayrı gösterilir |
| Paket yöneticisiyle kurulu Vexen | Güncelleme sahipliği korunur; doğrudan overwrite yapılmaz |

Her platform için test edilen artifact sürümü/hash'i, istemci sürümü ve gerçek kontrol sonucu kaydedilmeli. Mock CLI veya protokol test istemcisi gerekli alt kontrolleri sağlar; gerçek agent'ın ayarı yüklemesinin yerine geçmez.

## Somut sonraki uygulama ve araştırma sınırı

İlk değişiklik yeni updater seçmek değil: mevcut Rust `init`/`install` ve skill teslimini küçük bir setup içinde birleştirmek, kalıcı executable yolunu kullanmak, doğru başarı durumlarını göstermek. Sonra tek agent ve desteklenen işletim sistemlerinde gerçek kurulum testleri. Native plugin bootstrap çözüldüğünde kısa marketplace akışı yayımlanabilir.

Bu ek inceleme kaynak kodu ve dokümantasyon denetimidir. Kütüphaneler derlenmedi, kurucular çalıştırılmadı, platform garantisi veya dependency lisans uyumu için kapsamlı denetim yapılmadı. Bunlar uygulama seçiminde ayrıca doğrulanacak; burada yeni ürün yeteneği veya kusursuz kurulum sonucu iddia edilmiyor.
