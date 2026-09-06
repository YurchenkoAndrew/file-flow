use std::fs;
use std::path::{Path, PathBuf};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::LibraryExt;
use typst::World;

pub struct TypstSandbox {
    library: LazyHash<typst::Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    source: Source,
}

impl TypstSandbox {
    fn new(text: String, font_bytes: Vec<Vec<u8>>) -> Self {
        println!("   -> [Sandbox] Парсинг TTF шрифтов...");
        let fonts: Vec<Font> = font_bytes
            .into_iter()
            .flat_map(|b| Font::iter(Bytes::new(b)))
            .collect();
        println!("   -> [Sandbox] Шрифтов успешно загружено: {}", fonts.len());

        println!("   -> [Sandbox] Создание FontBook...");
        let book = FontBook::from_fonts(&fonts);

        println!("   -> [Sandbox] Подготовка исходного кода...");
        // Больше никакого хардкора с путями!
        // Source::detached сам создаст нужный виртуальный ID для файла.
        let source = Source::detached(text);

        println!("   -> [Sandbox] Сборка стандартной библиотеки Typst...");
        let library = typst::Library::default();

        println!("   -> [Sandbox] Песочница готова!");
        Self {
            library: LazyHash::new(library),
            book: LazyHash::new(book),
            fonts,
            source,
        }
    }
}

impl World for TypstSandbox {
    fn library(&self) -> &LazyHash<typst::Library> { &self.library }
    fn book(&self) -> &LazyHash<FontBook> { &self.book }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            // get_without_slash() отдает строку, PathBuf::from делает из нее нормальный путь
            Err(FileError::NotFound(PathBuf::from(id.vpath().get_without_slash())))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        println!("   [World] Typst запрашивает внешний файл: {:?}", id);
        Err(FileError::NotFound(PathBuf::from(id.vpath().get_without_slash())))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

pub struct PdfEngine;

impl PdfEngine {
    pub fn generate_from_template(
        typst_content: String,
        target_path: &Path,
        fonts_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        println!("⚙️ [PdfEngine] Чтение файлов шрифтов с диска...");
        let regular = fs::read(fonts_dir.join("Roboto-Regular.ttf"))?;
        let bold = fs::read(fonts_dir.join("Roboto-Bold.ttf"))?;

        println!("⚙️ [PdfEngine] Инициализация песочницы...");
        let world = TypstSandbox::new(typst_content, vec![regular, bold]);

        println!("⚙️ [PdfEngine] ЗАПУСК КОМПИЛЯЦИИ...");
        let compiled = typst::compile(&world);

        println!("⚙️ [PdfEngine] Компиляция завершена! Проверка на ошибки...");

        // Разбираем результат явно, чтобы вытащить ошибки синтаксиса в консоль
        let document = match compiled.output {
            Ok(doc) => {
                println!("✅ [PdfEngine] Синтаксис шаблона идеален!");
                doc
            },
            Err(errs) => {
                println!("\n❌❌❌ ОШИБКА СИНТАКСИСА В TYPST ШАБЛОНЕ ❌❌❌");
                println!("В шаблоне есть некорректные символы или сломана разметка:");
                for (i, err) in errs.iter().enumerate() {
                    println!("  {}. ОШИБКА: {}", i + 1, err.message);
                    // Итерируемся по вектору напрямую, без обертки Some()
                    for hint in &err.hints {
                        println!("     ПОДСКАЗКА: {:?}", hint);
                    }
                }
                println!("❌❌❌ ГЕНЕРАЦИЯ ОСТАНОВЛЕНА ❌❌❌\n");
                return Err("Синтаксическая ошибка в Typst шаблоне".into());
            }
        };

        println!("⚙️ [PdfEngine] Экспорт в PDF...");
        let pdf_bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
            .map_err(|errs| format!("Ошибка генерации PDF: {:?}", errs))?;

        println!("⚙️ [PdfEngine] Сохранение на диск...");
        fs::write(target_path, pdf_bytes)?;

        println!("✅ [PdfEngine] Документ успешно создан!");
        Ok(())
    }
}