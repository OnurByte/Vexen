> **Research notes and proposals, not shipped.** None of the projects below is integrated. Written in Turkish. Status: [roadmap](../roadmap.md).

# Vexen için GitHub araştırması: yapıya uygun güçlendirme alanları

Araştırma tarihi: 2 Kasım 2026. Bu belge öneridir; aşağıdaki projeler kurulmadı veya Vexen'e entegre edilmedi. Önceki gerçek-agent değerlendirmesinin kimlik doğrulama engeli devam ediyor; bu araştırma o deneyin yerine geçmez.

## Seçim ölçütleri

Mevcut kaynaklar: `Cargo.toml`, `src/cli/observe.rs`, `.github/workflows/release.yml`, `docs/specification.md` ve güncel güvenilirlik incelemesi. Uyum ölçütleri: tek Rust çalıştırılabilir dosyası, yerel SQLite/FTS5, kaynak ve test kanıtı, isteğe bağlı entegrasyon, arka planda servis veya zorunlu model olmaması.

Olgunluğu yalnızca yıldız sayısıyla değerlendirmedim. Proje geçmişini, yayımlanan sürümleri, bakım açıklamalarını, test/dokümantasyon varlığını ve Vexen'in gerçekten kullanabileceği arayüzü kontrol ettim. Aşağıdaki uyum ve öncelik değerlendirmeleri benim çıkarımımdır; entegrasyonun performansını kanıtlayan deneyler değildir.

## 1. Kanıtlı bellek için test raporu adaptörleri — yüksek ürün uyumu

Kaynak: [nextest-rs/nextest](https://github.com/nextest-rs/nextest), [JUnit çıktı belgesi](https://nexte.st/docs/machine-readable/junit/).

Olgunluk işareti: 2022'den beri var; kontrol edilen son GitHub sürümü `cargo-nextest-0.9.150`, 26 Ekim 2026. JUnit desteği ve ayrı test vakalarının raporlanması belgelenmiş.

Vexen bugün komutun çıkış kodunu ve çıktı özetini saklıyor. Önerim, açıkça çalıştırılan bir test raporundan seçilmiş test kimliği, sonucu, kaynak rapor özeti ve kod durumu bağını almak. Böylece "hangi test bu dersi destekliyor?" sorusu daha somut yanıtlanabilir. Nextest ilk Rust adaptörü için iyi bir referans; başka diller için aynı rapor yaklaşımı daha sonra değerlendirilebilir.

Önemli ayrım: yüklenen bir XML dosyası tek başına doğrulanmış komut yürütmesi değildir. İçe aktarılan beyan ile Vexen'in gözlemlediği çalıştırma ayrı tutulmalı. Nextest'in raporları başarısız test çıktısını varsayılan olarak içerebildiği için Vexen ham raporu veya stdout/stderr alanlarını belleğe kopyalamamalı. Test ile dosya ilişkisi de rapordan kendiliğinden çıkarılamaz.

İlk kabul ölçütü: tek bir gerçek çalıştırmada geçen, kalan ve yeniden denemede değişen testleri karıştırmadan kaydetmek; aynı raporun ikinci aktarımında kayıt çoğaltmamak. Efor: orta.

## 2. Agent'a ulaşan bilginin uçtan uca doğrulanması — hemen uygun

Kaynak: [modelcontextprotocol/inspector](https://github.com/modelcontextprotocol/inspector), [CLI smoke-test rehberi](https://github.com/modelcontextprotocol/inspector/blob/main/docs/cli-smoke-testing.md).

Olgunluk işareti: resmî MCP organizasyonu, 2024'ten beri proje geçmişi; kontrol edilen sürüm `2.13.0`, 28 Ekim 2026. CLI üzerinden stdio bağlantısı ve makinece okunabilir test akışı belgelenmiş.

Vexen'in Rust birim testlerine ek olarak bağımsız bir istemciyle bağlantı, araç keşfi ve gerçek kayıt okuma kontrolü yapılabilir. Özellikle yalnızca durum kodunu değil ders metnini de karşılaştırmak, son bulduğumuz metin yanıtı hatasının geri gelmesini önler.

Bu araç geliştirme/CI tarafında kalmalı; son kullanıcıya Node zorunluluğu getirmemeli. Inspector v2 geliştirme koşulu Node >=22.19.0; Vexen'in npm başlatıcısının Node 18 gereksinimiyle aynı şey değil. CI sürümü sabitlenmeli. Protokol bağlantısı testi, agent'ın görevi çözmesini kanıtlamaz.

İlk kabul ölçütü: boş, geçerli, eski ve çelişkili kayıt yanıtlarının hem metin hem yapılandırılmış biçimde doğrulanması. Efor: düşük–orta.

## 3. Bellek yaşam döngüsü için üretilmiş senaryo testleri — en düşük maliyetli sağlamlaştırma

Kaynak: [proptest-rs/proptest](https://github.com/proptest-rs/proptest), [paket sürümü ve lisansı](https://github.com/proptest-rs/proptest/blob/main/proptest/Cargo.toml).

Olgunluk işareti: 2017'den beri; README mimarinin uzun süredir büyük değişiklik görmediğini ve bakımın ağırlıkla pasif olduğunu açıkça belirtiyor. Kaynak manifestinde `1.12.0` görülüyor. GitHub'ın `releases/latest` uç noktası eski `0.9.6` kaydını döndürdüğünden bunu güncel paket sürümü olarak kullanmadım.

Sadece test bağımlılığı olarak kullanılabilir. Önerilen senaryolar: kayıt → başarısız sonuç → dosya değişikliği → dal değişikliği → çelişkili sonuç → supersede → export/import. Değişmezler: eski kaydı güncel diye sunmamak, çelişkiyi gizlememek, başka projeye taşmamak, üç öğe ve yanıt boyutu sınırını korumak.

İlk kabul ölçütü: bir hata bulunduğunda proptest'in küçülttüğü örneği kalıcı regresyon testine çevirmek. Son kullanıcı paketine çalışma zamanı yükü eklenmez. Efor: düşük–orta.

## 4. Dosya yerine kod parçasına bağlı bellek — yüksek farklılaşma, araştırma gerekli

Kaynak: [tree-sitter/tree-sitter](https://github.com/tree-sitter/tree-sitter), [ast-grep/ast-grep](https://github.com/ast-grep/ast-grep).

Olgunluk işareti: Tree-sitter 2013'ten beri; artımlı sözdizimi ağacı ve Rust bağları mevcut. Rust ile yazılmış ast-grep 2022'den beri; kontrol edilen sürümü `0.51.1`, 2 Kasım 2026. Kod yapısına göre arama sağlar.

Bugün bir dosyanın herhangi bir yeri değişince o dosyaya bağlı ders eskiyor. Niş fırsat: dersin ilgili fonksiyonunu ve yerel kod yapısını ayrıca tanımlamak. Örneğin dosyaya ilgisiz bir yardımcı fonksiyon eklenmesi, fiyat yuvarlama dersini yeniden incelemeye aday kılabilir.

Ancak aynı fonksiyon adı veya AST biçimi, dersin hâlâ geçerli olduğunu kanıtlamaz. Çağıran kod, bağımlılıklar ve iş kuralları değişebilir. İlk sürüm yalnızca "yeniden doğrulanabilir eski ders adayı" göstermeli; mevcut kesin içerik kontrolünü sessizce gevşetmemeli. Başlangıç için tek dilde salt okunur deney öneririm. Otomatik kod yazma kapsamına girilmemeli.

İlk kabul ölçütü: ilgisiz değişikliklerde geri kazanılan yararlı dersler ile anlamı değişmiş koda verilen yanlış hatırlatmaların birlikte ölçülmesi. Efor: yüksek.

## 5. İstemci ayarlarının sürümlü bilgi tabanı — iyi referans, doğrudan çekirdek bağımlılığı değil

Kaynak: [neon-solutions/add-mcp](https://github.com/neon-solutions/add-mcp), [test komutları](https://github.com/neon-solutions/add-mcp/blob/main/package.json).

Olgunluk: Ocak 2026 başlangıçlı, daha genç bir proje. `v2.6.1` sürümü 1 Kasım 2026'da yayımlanmış; istemci yapılandırmaları ve kurulum testleri var. Uzun geçmişli bir altyapı kütüphanesiyle aynı güven düzeyinde değerlendirmiyorum.

Vexen için en yararlı kısmı istemci yolları, proje/global kapsam farkları ve format uç durumları. Örneğin Copilot CLI ile Claude'un ortak `.mcp.json` dosyası kullanımı ve OpenCode'un farklı yerleşimleri test fikri sağlayabilir. Her ayrıntı ilgili istemcinin resmî belgesiyle ayrıca doğrulanmalı.

Rust çekirdeğini Node'a bağlamak yerine, sürümü sabitlenmiş test örneklerinden öğrenmek daha uygun. Projenin bazı yolları JSON yorumlarını düşürebiliyor ve otomatik onay ayarlarını değiştirme seçenekleri var; Vexen'in koruma ve izin tercihleri aynen muhafaza edilmeli.

İlk kabul ölçütü: bir istemci eklerken ikinci kurulumun aynı sonucu vermesi, kaldırmanın yalnızca Vexen girdisini silmesi ve başka ayarları bozmaması. Efor: orta.

## 6. Kurulum ve sürüm dağıtımı — olgun ama koşullu değer

Kaynak: [axodotdev/cargo-dist](https://github.com/axodotdev/cargo-dist).

Olgunluk işareti: 2022'den beri; kontrol edilen `v0.34.0` sürümü 2 Kasım 2026. Proje artık kendisini `dist` olarak adlandırıyor; paket ve depo adı cargo-dist olarak da geçiyor. Çoklu platform binary, arşiv, kurucu ve dağıtım manifesti üretimi belgelenmiş.

Vexen'in zaten beş hedef için derleme, sağlama toplamı ve npm yayımlama akışı var. Dolayısıyla kazanç "ilk defa release yapmak" değil, bu özel bakım yükünü azaltmak olabilir. Mevcut arşiv adları ve npm başlatıcısının indirme sözleşmesi karşılaştırılmadan geçiş önerilmez. Skill'in `vexen` beklemesiyle npx kurulumu arasındaki uyumsuzluk da bu araç eklenince kendiliğinden çözülmez.

İlk kabul ölçütü: mevcut platformlarda aynı kurulum sözleşmesi ve doğrulama davranışı; ek paket yöneticisi bakımının gerçekten azalması. Efor: orta; şimdiki ilk öncelik değil.

## Şimdilik ana yola almadıklarım

- [MCP conformance](https://github.com/modelcontextprotocol/conformance): resmî ve faydalı; incelenen sunucu testleri URL üzerinden çalışıyor. Stdio Vexen'e doğrudan takılabilen bir drop-in test diye sunmuyorum. Önce Inspector; conformance için desteklenen bir taşıma adaptörü veya uygun senaryo eşlemesi gerekir.
- [adrkit](https://github.com/mbeacom/adrkit): makinece okunur kararlar ve `affects` ilişkisi güzel bir birlikte çalışma fikri. Temmuz 2026 başlangıçlı olması ve önerilen birlikte çalışma yönünün hâlâ değerlendirmede bulunması nedeniyle önce sınırlı ADR içe aktarma araştırması; karar yönetişimini Vexen içinde yeniden kurmak değil.
- [Engram](https://github.com/Gentleman-Programming/engram): yerel SQLite/FTS5 ve tek binary yaklaşımı bakımından yakın ürün referansı. Kurulum ve ilk kullanım akışını incelemek değerli; ikinci bir bellek motorunu Vexen'in içine koymak için gerekçe değil.
- [Gitleaks](https://github.com/gitleaks/gitleaks): gizli bilgi örnekleri ve isteğe bağlı geliştirme taraması için referans olabilir. Bakımcı yeni özellikleri kapatıp güvenlik yamalarıyla sınırladığını açıklıyor; kaynak sayfası artık Betterleaks'e yönlendiriyor. Bu nedenle yeni çekirdek entegrasyon için ilk önerim değil.
- Zorunlu embedding, vektör veritabanı ve ayrı bellek sunucusu: şu anki çevrimdışı, küçük ve bağımsız ürün sözleşmesine ek yük getirir; mevcut başarısızlıkların çözümü olduğu gösterilmedi.

## Benim sıralamam

1. Inspector ile bağımsız yanıt doğrulaması ve proptest ile yaşam döngüsü testleri.
2. Nextest/JUnit üzerinden açık kaynak bağıyla yapılandırılmış test kanıtı.
3. add-mcp örneklerinden sürümlü istemci testlerini genişletme.
4. Tree-sitter/ast-grep ile tek dilli, ölçülen kod-parçası belleği deneyi.
5. Mevcut dağıtım bakım yükü gerekçelendirirse cargo-dist geçişi.

En güçlü ürün nişi: **hangi yaklaşımın, hangi kod durumunda, hangi testle başarısız olduğunu bilen ve kod değişince bu dersi yeniden doğrulamaya aday gösterebilen yerel bellek**. Yukarıdaki projeler bu fikrin ayrı parçalarını güçlendirebilir; hiçbirinin eklenmesi tek başına yüzde yüz agent başarısı sağlamaz.


## Ana araştırma kaydıyla ilişki

Bu adayların öncelik, entegrasyon sınırı, kabul kanıtı ve mevcut karar durumları [ana güvenilirlik araştırmasının yol haritasına](../memory-reliability-investigation.md#github-research-architecture-compatible-strengthening-roadmap) işlendi. Bu dosya kaynak araştırmasıdır; ana kayıt ise doğrulanan düzeltmelerle gelecekteki önerileri ayrı tutar. Hiçbir aday henüz uygulanmış veya kabul edilmiş bağımlılık sayılmamalıdır.


## Güncel kurulum yönü: Rust öncelikli

Sonraki [Rust-native kurulum değerlendirmesi](rust-native-installation.md), kurulumun iş mantığını Rust içinde birleştirmeyi esas alır. `add-mcp` istemci formatı referansı olarak kalır; Node tabanlı kurulum motoru önerilmez. `cargo-dist` dağıtımı, mevcut Rust kodu ise agent kaydı ve skill/proje başlangıcını üstlenebilir. Genç ve arşivlenmiş örneklerin durumları yeni belgede açıkça ayrılmıştır.


Rust kurulum belgesinin [derin inceleme bölümüne](rust-native-installation.md#derin-inceleme-kurulumun-eksik-kalan-teknik-sınırları) ek kaynaklar işlendi: `self_update`, `self-replace`, `axoupdater`, cargo-binstall atomik dosya/kilit modülleri ve Rust `include_str!`. Bunlar birbirine eklenecek bir dependency listesi değildir. Kaynak commit'leri, güvenlik varsayımları, Windows/eşzamanlı setup sınırları ve kabul testleri aynı belgede tutulur.
