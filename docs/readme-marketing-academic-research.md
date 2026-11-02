# Vexen README pazarlaması: akademik dayanak ve uygulama önerileri

Araştırma tarihi: 2 Kasım 2026. Durum: kaynak incelemesi ve öneri; kullanıcı deneyi yapılmadı. README ve görsel dosyaları bu araştırma sırasında değiştirilmedi.

## Amaç ve değişmeyecek tasarım dili

Amaç, AI kodlama araçlarını kullanan ama MCP, bellek mimarisi veya araştırma metodolojisi bilmeyen bir ziyaretçinin üç soruyu yanıtlayabilmesi: Vexen ne yapıyor, bana ne zaman yarar, nasıl denerim?

**Oxocarbon sabit.** `brand.md` içindeki `#161616`, `#262626`, `#f2f4f8`, `#78a9ff`, `#a6c8ff` renkleri; Geist/Geist Mono ailesi; mevcut logo ve sakin kart yaklaşımı korunacak. Öneriler metin, bilgi sırası, açıklama hiyerarşisi ve okunurlukla ilgili. Yeni tema, dekoratif efekt veya yeni renk sistemi önerilmiyor.

Akademik çalışmalar, Vexen'in performansını veya bu README'nin kurulum oranını kanıtlamaz. Aşağıda araştırma bulgusu, incelenen mevcut durum ve ürün önerisi ayrı tutuluyor. Bu inceleme sistematik literatür taraması veya meta-analiz değildir; belirli iletişim sorularına yönelik kaynak seçimidir.

## Akademik kaynaklar ve neye dayanak oldukları

### 1. README'de amaç ve kullanım yolunu görünür kılmak

Prana, Treude, Thung, Atapattu ve Lo — *Categorizing the Content of GitHub README Files*, Empirical Software Engineering; 2018 çevrimiçi yayın, 2019 cilt yayını. DOI: `10.1007/s10664-018-9660-3`.

**Bulgu:** 393 depodan 4.226 bölüm sınıflandırıldı. Ne ve nasıl bilgilerinin yaygın, amaç ve durum bilgilerinin daha az bulunduğu raporlandı. Bölüm etiketleriyle yapılan küçük değerlendirmede 20 yazılım profesyonelinin çoğu bilgi bulmanın kolaylaştığını düşündü. [Üniversite yayın kaydı](https://ink.library.smu.edu.sg/sis_research/4360/) · [Yazarların makalesi](https://soarsmu.github.io/papers/2019/Prana2019CategorizingGithubReadme.pdf).

**Sınır:** Açık kaynak benimsenmesi veya yeni başlayanların kurulum başarısı üzerinde nedensel bir deney değil. Çok sayıda rozet eklemek gerektiğini göstermez.

**Vexen çıkarımı:** Görünür sıra “hangi sorunu çözer → nasıl görünür → nasıl denenir → hangi kanıt var” olmalı. Erken geliştirme durumu saklanmamalı. Başlıklar okuyucunun sorusunu söylemeli; iç mimariyi öğrenmesi beklenmemeli.

### 2. Gereksiz ağır kelimeler uzmanlık izlenimini güçlendirmeyebilir

Oppenheimer — *Consequences of Erudite Vernacular Utilized Irrespective of Necessity: Problems with Using Long Words Needlessly*, Applied Cognitive Psychology 20, 139–156, 2006. DOI: `10.1002/acp.1178`.

**Bulgu:** Çalışma, gereksiz kelime karmaşıklığının yazarın zekâsına ilişkin değerlendirmeleri olumsuz etkileyebildiğini ve işleme akıcılığının rolünü inceliyor. [Yayıncı özeti](https://onlinelibrary.wiley.com/doi/10.1002/acp.1178?afsrc=1).

**Sınır:** Ölçülen sonuç doğrudan README anlama, ürün kalitesi veya kurulum dönüşümü değil. Bu kaynak her teknik terimin kaldırılması gerektiğini kanıtlamaz. Bu incelemede yayıncı özeti ve erişilebilir makale metni indeksleri kullanıldı; yayıncı tam metnine erişim kısıtlıydı.

**Vexen çıkarımı:** İlk ekranda “evidence-backed repository memory”, “abstention”, “interop” gibi kavramlar yerine “save a failed fix”, “check what happened”, “keep notes with your project” kullanılabilir. Gerekli teknik ad, kullanıcı ilgili adıma geldiğinde kısa açıklamayla verilmeli.

### 3. Somut anlatım, ihtiyacın anlaşıldığını hissettirebilir

Packard ve Berger — *How Concrete Language Shapes Customer Satisfaction*, Journal of Consumer Research 47(5), 787–806, 2021; çevrimiçi 2020. DOI: `10.1093/jcr/ucaa038`.

**Bulgu:** Müşteri hizmetleri araştırmalarında somut dil, konuşanın ihtiyacı dinlediği/anladığı algısıyla ve müşteri tepkileriyle ilişkilendiriliyor; deneysel çalışmalarda yanıtın ihtiyaca uygunluğu önemli. [Makale ve yayıncı indeksli metni](https://academic.oup.com/jcr/article/47/5/787/5873524).

**Sınır:** İnsan müşteri hizmetleri konuşmaları, kendi kendine okunan geliştirici README'siyle aynı bağlam değil. Satış veya memnuniyet yüzdeleri Vexen'e taşınamaz. Yayıncı sayfasının tam metin erişimi bu oturumda kısıtlı; yayıncıdan indekslenen özet ve ilgili çalışma bölümleri esas alındı.

**Vexen çıkarımı:** “Agents retain useful context” yerine “Monday's failed fix is available for Friday's chat” gibi bir olay anlatmak değerlendirilebilir. Örnekler gerçek görev tanımlarından veya açıkça etiketlenmiş temsili senaryolardan gelmeli; hayalî müşteri başarısı kullanılmamalı.

### 4. Görsel ve metnin ilişkisini işaretlemek, ön bilgisi düşük kişiler için yararlı olabilir

Richter, Scheiter ve Eitel — *Signaling Text-Picture Relations in Multimedia Learning: A Comprehensive Meta-analysis*, Educational Research Review 17, 19–36, 2016. DOI: `10.1016/j.edurev.2015.12.003`.

**Bulgu:** 27 çalışma, 45 karşılaştırma ve 2.464 katılımcıyı kapsayan meta-analizde metin-görsel karşılıklarını işaretleme için küçük–orta olumlu etki raporlandı; ön bilgisi düşük öğrenenler daha fazla yarar gördü. [Yayıncı özeti](https://www.sciencedirect.com/science/article/pii/S1747938X15000664) · [Yazar kurumu kaydı](https://publikationen.uni-tuebingen.de/xmlui/handle/10900/75800).

**Sınır:** Eğitim materyallerini inceler; belirli bir renk paletinin üstünlüğünü veya pazarlama dönüşümünü göstermez. Tam metin yayıncı erişimi kısıtlı olduğundan örneklem ve sonuç özetteki kapsamla sınırlı aktarıldı.

**Vexen çıkarımı:** Oxocarbon mavisini tutarlı olarak kaydedilen ders ve sonraki oturuma aktarım için kullanmak; önce/sonra kartlarında aynı gün/araç/eylem konumlarını eşlemek; gerekli etiketleri ilgili öğenin yanına koymak. Mavi renk tek başına anlam taşımamalı; metin etiketi de bulunmalı.

### 5. Aynı anda çok fazla açıklama anlama işini zorlaştırabilir

Mayer ve Moreno — *Nine Ways to Reduce Cognitive Load in Multimedia Learning*, Educational Psychologist 38(1), 43–52, 2003. DOI: `10.1207/S15326985EP3801_6`.

**Bulgu:** Çoklu ortam öğrenmesi çerçevesinde sınırlı işleme kapasitesi ve farklı aşırı yük durumlarına yönelik düzenleme yöntemleri ele alınıyor. [Üniversitede barındırılan makale tam metni](https://www.psychology.mcmaster.ca/bennett/psy720/readings/m1/m1r4.pdf).

**Sınır:** Çok sayıda örnek anlatımlı eğitim/animasyon bağlamındadır. “Kart sayısı üç olmalı” veya “görseldeki metin Markdown'da tekrarlanmamalı” sonucu doğrudan çıkarılamaz. Erişilebilir metin ve alt açıklamalar korunmalı.

**Vexen çıkarımı:** Aynı ana bölümde görev başarısı, eski kayıt eleme, istemci bağlantısı ve başarısız dosya tahmini araştırmasını art arda açıklamak yerine, ana fayda ve ileri incelemeye farklı okuma derinlikleri vermek. Eklenen kartlar korunabilir; ayrıntılı olanlar açılır bölümde veya Research sayfasında yer alabilir.

### 6. Sonucun sınırını açıklamak otomatik olarak güveni yok etmez

van der Bles, van der Linden, Freeman ve Spiegelhalter — *The Effects of Communicating Uncertainty on Public Trust in Facts and Numbers*, PNAS 117(14), 7672–7683, 2020. DOI: `10.1073/pnas.1913678117`.

**Bulgu:** Beş deneyde toplam 5.780 katılımcı; sayılar ve kaynak güvenilirliğinde küçük azalmalar, çoğunlukla sözel belirsizlik iletişiminde gözlendi. Sayısal belirsizlik iletişiminin güvene etkisi sınırlıydı. [Açık erişimli makale](https://pmc.ncbi.nlm.nih.gov/articles/PMC7149229/).

**Sınır:** Haber ve istatistik iletişimi; ticari ürün tanıtımı değil. Bu çalışma “sınırlılık yazmak güveni mutlaka artırır” veya “her uyarı zararsızdır” demiyor. Sadece örneklem büyüklüğü yazmak da gerçek belirsizliği aktarmaya yetmez.

**Vexen çıkarımı:** 11/20 ve 2/20 sonucu, kısa ve yakındaki yöntem/sınır açıklamasıyla verilmeli. Tam metodoloji bağlantıda kalabilir; ana iddianın sınırı bağlantının arkasına saklanmamalı. Küçük sentetik pilot, genel performans vaadine dönüştürülmemeli.

## Mevcut README'nin somut değerlendirmesi

Bunlar kullanıcı deneyi bulgusu değil, mevcut dosyanın kaynaklara dayanılarak yapılan içerik incelemesidir.

| Mevcut öğe | Güçlü taraf | Yeni ziyaretçi açısından olası sorun | Öneri |
|---|---|---|---|
| “Your AI agent forgot what failed. Vexen didn't.” | Akılda kalıcı sorun anlatımı | Önceden kaydetme gereğini okumadan otomatik hafıza sanılabilir | Hemen altındaki cümlede kaydetme ve sonraki agent'ın kontrol etmesi açık kalsın |
| Önce/sonra kartı | Aynı sorunu iki oturumla somutlaştırıyor | Dersin tam olarak ne olduğu görünmüyor | Tek cümlelik örnek ders eklemeyi test et; temsili etiketi koru |
| Üç fayda kartı | Sade ve birbirinden ayrılan faydalar | “Proof” yeni ziyaretçi için soyut kalabilir | “See the file and test behind each note” gibi açıklayıcı alt metin |
| Kurulum | İki işletim sistemi yolu ve gerçek komutlar var | Skill/PATH/MCP açıklaması ilk denemeyi kesintiye uğratabiliyor | Ana deneme yolunu görünür tut; alternatifleri açıkça etiketlenmiş ayrıntıya ayır |
| İlk kontrolün boş dönmesi | Doğru beklentiyi açıklıyor | Ürünün değerini deneyimletmeden “bir şey bulunamadı” ile bitiyor | Gerçek çıktılı, ayrı bir demo projesi öner; normal kullanıcının projesine sahte hata yazma |
| Çok sayıda istemci logosu | Araç seçimini tanıdıklaştırıyor | Kurulum, keşif ve gerçek görev desteği aynı sanılabilir | İlk bakışta Claude/Codex/Cursor başlangıç bağlantıları; kapsam açıklaması ve geniş liste ayrıntıda |
| Does it help? | Kaynaklı iç sonuç ve sınırlılık var | Dört ayrı araştırma türünü okuyucu kendisi ayırıyor | Ana pilot + yanında kısa koşullar; ek kartlar açık etiketli ileri inceleme katmanında |
| “with relevant Vexen memory” | Kontrolün amacını anlatıyor | Her enabled koşuda kayıt döndüğü sanılabilir | “with Vexen enabled and pre-recorded notes” tercih et; 12/20 son-yanıt ayrıntısı metodolojide kalsın |

### Mobil okunurluk: ölçülebilen bir risk

Kartlar 1.200 birimlik SVG genişliğine sahip. Mevcut ek araştırma kartlarında 17 birimlik alt açıklamalar var. Görsel 360 CSS piksel genişlikte gösterilirse geometrik karşılığı `17 × 360 / 1200 = 5,1` CSS piksel olur. Bu bir kullanıcı okuma testi veya canlı GitHub ekran ölçümü değil; doğrudan küçültme hesabıdır.

Bu nedenle kartların yalnızca tam boy masaüstü PNG önizlemesi yeterli değil. 360/390/768 piksel sayfa genişliklerinde gerçek GitHub görünümü kontrol edilmeli. Çözüm renk değiştirmek değil: daha az metin, uygun yerde tek sütun, gerekirse aynı Oxocarbon dilinde dar ekran görseli ve kritik koşulların normal Markdown metninde bulunması. Alt metni veya sınırlılık açıklamasını silmek çözüm sayılmamalı.

## Önerilen okuma sırası

Bu sıra araştırmalardan çıkarılmış ürün hipotezidir; akademik olarak kanıtlanmış bir dönüşüm şablonu değildir.

1. **Sorun ve fayda:** yeni oturumda aynı başarısız yaklaşımı yeniden açıklama ihtiyacı.
2. **Tek somut önce/sonra:** başarısız deneme → kaydedilen kısa ders → sonraki agent'ın kontrolü.
3. **Başlangıç:** gereksinimler, işletim sistemine uygun komutlar, yeniden başlatma ve beklenen yanıt.
4. **Gerçek değer gösterimi:** ayrı demo projesinde kaydın ve test bağlantısının görülmesi; gerçek sonuç henüz yoksa yalnızca temsili olarak etiketlenmiş örnek.
5. **Ana kanıt:** 11/20 vs 2/20; yanında koşullar ve sınır; doğrudan kaynak.
6. **İleri inceleme:** mevcut ek araştırma kartları, tüm istemciler, ham veriler ve başarısız araştırma. Bunlar görünür açılır başlıklarla erişilebilir kalmalı.
7. **Kontrol ve mahremiyet:** yerel depolama, nelerin kaydedildiği, agent'ın kendi veri politikaları ve erken geliştirme durumu.

## Kullanılabilir metin önerileri

Bunlar Vexen için yazılmış taslaklardır; araştırmadan alıntı değildir. README henüz bu taslaklarla değiştirilmedi.

### İlk ekran

```text
Your AI agent forgot what failed. Vexen didn't.

Save failed fixes and important decisions with your project.
Your next coding agent can check those notes before trying again.

Free & open source · Local storage · No account

Get started
```

Mevcut akılda kalıcı başlık korunabilir. Alt cümleler bunun otomatik sohbet kaydı olmadığını eylemler üzerinden anlatır. Sonraki agent'ın gerçekten kontrol etmesi gerektiği gizlenmez.

### Kart metin standardı

| Kart | Başlık | Alt açıklama |
|---|---|---|
| Fayda | Keep the lesson. | Save what failed for the next coding session. |
| Kaynak | See what happened. | Check the file and test behind a saved note. |
| Kontrol | Keep it local. | Notes stay with your project. No Vexen account. |

Her kart tek soruyu yanıtlamalı. Araştırma kartları ayrı bir standart taşımalı: sorusu → ölçülen sonuç → koşullar → sınır → kaynak. “Tested connections” ile “solved tasks” aynı başarı türü gibi gösterilmemeli.

### Araştırma sonucu

```text
Can a saved lesson help on a coding task?

In our small internal pilot, the agent passed 11 of 20 targeted runs
with Vexen enabled and pre-recorded notes, versus 2 of 20 without memory.

Five synthetic tasks. One agent/model. Explicit instructions to check memory.
This does not establish general coding improvement.

Read the experiment and raw data
```

“5.5× smarter” gibi oranlar, “never repeat mistakes” gibi garantiler veya ayrı çalışma gruplarını birleştiren toplam başarı skoru kullanılmamalı.

### İlk kullanım için beklenti cümlesi

```text
A new project has no saved lessons yet. “No relevant evidence” is normal.
After a failed test, ask your agent to save what happened and its test result.
```

Demo ihtiyacı, uygulama iş listesidir. Gerçek çalıştırma ve çıktısı olmadan bu notta önerilen demo “hazır” diye sunulmamalı. Mevcut npx/skill komut farkı çözülmeden “one command, everything ready” iddiası eklenmemeli.

## Nasıl doğrulayacağız?

Önce aynı Oxocarbon görsel sistemiyle iki içerik düzenini karşılaştır: mevcut README ve yukarıdaki sadeleştirilmiş sıra. İlk turda 6–8 hedef kullanıcıyla biçimlendirici kullanılabilirlik testi öneriyorum. Bu sayı istatistiksel etki ispatı değil, ilk sorunları bulmak için çalışma tercihidir. Katılımcılar AI kodlama aracı kullanmış, Vexen ve MCP belleğine aşina olmayan kişiler olmalı.

İlk maruziyette grupları ayır; aynı kişiye önce bütün ürünü açıklayıp sonra “ilk kez anlama” testi yapma. Araç, terminal/Git deneyimi ve mobil/masaüstü durumu kaydedilsin. Kimseyi “normie” diye etiketlemek yerine ön bilgiyi tarif et.

| Görev | Ölçüm | Hangi kararı etkiler? |
|---|---|---|
| İlk kısa bakıştan sonra ürünü kendi sözleriyle anlat | Kaydetme, sonraki oturum ve agent'ın kontrolü doğru mu? | İlk ekran ve örnek |
| “Sohbetlerim kendiliğinden kaydediliyor mu?” | Otomatik kayıt yanılgısı var mı? | Başlık altı açıklama |
| Kendi aracının başlangıcını bul | Yardım almadan doğru bölüme ulaşma, takıldığı terimler | Bağlantılar ve kurulum sırası |
| Yeni projede boş yanıtı yorumla | Ürünü bozuk mu sanıyor, henüz ders olmadığını mı anlıyor? | İlk kullanım mesajı |
| Deney sonucunu anlat | Küçük pilotu genel yüzde 55 başarı garantisi sanıyor mu? | Kanıt kartı ve sınırlılıklar |
| Dar ekranda kartı oku | Başlık, ders ve sınır açıklaması yakınlaştırmadan okunabiliyor mu? | Kart yoğunluğu ve yerleşim |

10 saniyelik bakış veya iki dakikalık başlangıç bulma gibi süreler ancak ürün hedefi olarak kullanılabilir; bu makalelerden türeyen evrensel eşikler değildir. Kurulum süresi raporlanırken indirme, ağ ve hesap sorunlarını metin bulma süresinden ayır.

İlk turda başarıyı gerçek sayılarla raporla: örneğin “8 kişiden 5'i kaydetme gereğini doğru anlattı”. Küçük örneklemden yüzde dönüşüm artışı, anlamlılık veya genel pazar sonucu çıkarma. Daha büyük bir karşılaştırmada önceden belirlenmiş ölçüm ve uygun örneklem planı gerekir. Testler gönüllü olsun; ürünün telemetrisiz yapısı değiştirilmesin.

## Uygulama önceliği ve sınırlar

**Önce:** otomatik bellek yanılgısını azaltan alt metin; koşul düzeyinde doğru deney ifadesi; ana/ileri araştırma ayrımı; mobil kart okunurluğu; kullanıcı dilinde başlangıç bağlantıları.

**Sonra:** kaydı ve gerçek test kanıtını gösteren ayrı demo; ilk kullanım akışındaki npx/skill farkının giderilmesi; küçük kullanıcı testi ve bulgulara göre içerik düzenlemesi.

**Sabit kalanlar:** Oxocarbon paleti, mevcut logo, Geist yazı sistemi, ücretsiz/yerel ürün sözleşmesi, kaynaklara erişim ve deney sınırlılıkları. Yeni kart sayısı, daha iddialı slogan veya daha çok akademik kaynak tek başına daha iyi pazarlama ölçütü değildir.

Bu araştırmanın tamamlanması; kaynakların doğrulanması, önerilerin güncel README ile eşlenmesi, tasarım kısıtının korunması ve test planının yazılmasıdır. README'nin anlaşılabilirliğinin gerçekten arttığı iddiası ise kullanıcı değerlendirmesinden önce doğrulanmış sayılmaz.
