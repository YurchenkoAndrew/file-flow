import { Component, inject, input, OnInit, signal } from '@angular/core';
import { FormBuilder, FormControl, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { DocumentGeneratorService } from '../../../../services/document-generator.service';
import { DateHelper } from '../../../../helpers/date.helper';
import { MatFormField, MatInput, MatLabel } from '@angular/material/input';
import { MatButton } from '@angular/material/button';
import { MatTabsModule } from '@angular/material/tabs';
import { debounceTime } from 'rxjs';
import { QuillEditorComponent } from 'ngx-quill';
import { SafeHtmlPipe } from '../../../../pipes/safe-html-pipe';
import {MatOption, MatSelect} from "@angular/material/select";

export interface AgreementTemplateData {
    agreement_number: string;
    city: string;
    date: string;

    // Заказчик / Сторона 1
    customer_name: string;
    customer_position: string;
    customer_signatory_name: string;
    customer_genitive: string;
    customer_basis: string;
    customer_reg_info: string;         // Доп. инфо (номер, дата) - опционально
    customer_address: string;
    customer_id_type: 'БИН' | 'ИИН';   // Выбор БИН или ИИН
    customer_iin_bin: string;
    customer_kbe: string;              // Опционально
    customer_iik: string;
    customer_bank: string;
    customer_bik: string;

    // Подрядчик / Сторона 2
    contractor_name: string;
    contractor_position: string;
    contractor_signatory_name: string;
    contractor_genitive: string;
    contractor_basis: string;
    contractor_reg_info: string;       // Доп. инфо (номер, дата) - опционально
    contractor_address: string;
    contractor_id_type: 'БИН' | 'ИИН'; // Выбор БИН или ИИН
    contractor_iin_bin: string;
    contractor_kbe: string;            // Опционально
    contractor_iik: string;
    contractor_bank: string;
    contractor_bik: string;

    preamble_closing: string;
    body_text: string;
}

export interface AgreementFormModel {
    agreement_number: FormControl<string>;
    city: FormControl<string>;
    date: FormControl<string>;

    customer_name: FormControl<string>;
    customer_position: FormControl<string>;
    customer_signatory_name: FormControl<string>;
    customer_genitive: FormControl<string>;
    customer_basis: FormControl<string>;
    customer_reg_info: FormControl<string>;
    customer_address: FormControl<string>;
    customer_id_type: FormControl<'БИН' | 'ИИН'>;
    customer_iin_bin: FormControl<string>;
    customer_kbe: FormControl<string>;
    customer_iik: FormControl<string>;
    customer_bank: FormControl<string>;
    customer_bik: FormControl<string>;

    contractor_name: FormControl<string>;
    contractor_position: FormControl<string>;
    contractor_signatory_name: FormControl<string>;
    contractor_genitive: FormControl<string>;
    contractor_basis: FormControl<string>;
    contractor_reg_info: FormControl<string>;
    contractor_address: FormControl<string>;
    contractor_id_type: FormControl<'БИН' | 'ИИН'>;
    contractor_iin_bin: FormControl<string>;
    contractor_kbe: FormControl<string>;
    contractor_iik: FormControl<string>;
    contractor_bank: FormControl<string>;
    contractor_bik: FormControl<string>;

    preamble_closing: FormControl<string>;
    body_text: FormControl<string>;
}

export interface PreviewPage {
    pageNumber: number;
    showHeader: boolean;
    paragraphs: string[];
    showSignatures: boolean;
}

@Component({
    selector: 'app-agreement-template',
    imports: [
        ReactiveFormsModule,
        MatFormField,
        MatLabel,
        MatInput,
        MatButton,
        MatTabsModule,
        QuillEditorComponent,
        SafeHtmlPipe,
        MatSelect,
        MatOption,
    ],
    templateUrl: './agreement-template.html',
    styleUrl: './agreement-template.css',
})
export class AgreementTemplate implements OnInit {
    private fb = inject(FormBuilder);
    private docGenService = inject(DocumentGeneratorService);

    format = input.required<string>();
    form!: FormGroup<AgreementFormModel>;
    isGenerating = signal(false);
    pages = signal<PreviewPage[]>([]);

    quillModules = {
        toolbar: [
            ['bold', 'italic', 'underline'],
            [{ 'header': [1, 2, 3, false] }],
            [{ 'list': 'ordered' }, { 'list': 'bullet' }],
            [{ 'align': [] }],
            ['clean']
        ]
    };

    defaultData: AgreementTemplateData = {
        agreement_number: '1',
        city: 'г. Алматы',
        date: DateHelper.today(),

        // Заказчик
        customer_name: 'ТОО «Nova Development»',
        customer_position: 'Генеральный директор',
        customer_signatory_name: 'Ахметов Б.С.',
        customer_genitive: 'генерального директора Ахметова Б.С.',
        customer_basis: 'Устава',
        customer_reg_info: '',
        customer_address: 'г. Алматы, пр. Абая, д. 52, офис 301',
        customer_id_type: 'БИН',
        customer_iin_bin: '180440025910',
        customer_kbe: '17',
        customer_iik: 'KZ459260100192837465',
        customer_bank: 'АГФ АО «Банк ЦентрКредит»',
        customer_bik: 'KCJBKZKX',

        // Подрядчик
        contractor_name: 'ИП «Смирнов Д.А.»',
        contractor_position: 'Индивидуальный предприниматель',
        contractor_signatory_name: 'Смирнов Д.А.',
        contractor_genitive: 'Смирнова Д.А.',
        contractor_basis: 'свидетельства о государственной регистрации',
        contractor_reg_info: '№ 0009842 от 14.03.2018 г.',
        contractor_address: 'г. Алматы, ул. Толе би, д. 140, кв. 25',
        contractor_id_type: 'ИИН',
        contractor_iin_bin: '850315302194',
        contractor_kbe: '19',
        contractor_iik: 'KZ82722C000021345678',
        contractor_bank: 'АО «БТА Банк»',
        contractor_bik: 'ABKZKZKX',

        preamble_closing: 'заключили настоящий договор подряда о нижеследующем:',
        body_text: `<p><strong>1. ПРЕДМЕТ ДОГОВОРА</strong></p>
<p>1.1. Подрядчик обязуется по заданию Заказчика в установленный Договором срок выполнить работы по разработке веб-сайта, а Заказчик обязуется принять результат Работ и уплатить обусловленную Договором цену.</p>
<p><br></p>
<p><strong>2. СРОКИ И СТОИМОСТЬ РАБОТ</strong></p>
<p>2.1. Стоимость работ по Договору составляет 350 000 (триста пятьдесят тысяч) тенге.</p>
<p>2.2. Заказчик выплачивает предоплату в размере 50% в течение 3 банковских дней с момента подписания Договора. Оставшаяся сумма выплачивается в течение 3 банковских дней после подписания Акта сдачи-приемки.</p>`
    };

    ngOnInit(): void {
        this.form = this.fb.group<AgreementFormModel>({
            agreement_number: new FormControl(this.defaultData.agreement_number, { nonNullable: true, validators: [Validators.required] }),
            city: new FormControl(this.defaultData.city, { nonNullable: true, validators: [Validators.required] }),
            date: new FormControl(this.defaultData.date, { nonNullable: true, validators: [Validators.required] }),

            // Заказчик
            customer_name: new FormControl(this.defaultData.customer_name, { nonNullable: true, validators: [Validators.required] }),
            customer_position: new FormControl(this.defaultData.customer_position, { nonNullable: true, validators: [Validators.required] }),
            customer_signatory_name: new FormControl(this.defaultData.customer_signatory_name, { nonNullable: true, validators: [Validators.required] }),
            customer_genitive: new FormControl(this.defaultData.customer_genitive, { nonNullable: true, validators: [Validators.required] }),
            customer_basis: new FormControl(this.defaultData.customer_basis, { nonNullable: true, validators: [Validators.required] }),
            customer_reg_info: new FormControl(this.defaultData.customer_reg_info, { nonNullable: true }),
            customer_address: new FormControl(this.defaultData.customer_address, { nonNullable: true, validators: [Validators.required] }),
            customer_id_type: new FormControl(this.defaultData.customer_id_type, { nonNullable: true, validators: [Validators.required] }),
            customer_iin_bin: new FormControl(this.defaultData.customer_iin_bin, { nonNullable: true, validators: [Validators.required] }),
            customer_kbe: new FormControl(this.defaultData.customer_kbe, { nonNullable: true }),
            customer_iik: new FormControl(this.defaultData.customer_iik, { nonNullable: true, validators: [Validators.required] }),
            customer_bank: new FormControl(this.defaultData.customer_bank, { nonNullable: true, validators: [Validators.required] }),
            customer_bik: new FormControl(this.defaultData.customer_bik, { nonNullable: true, validators: [Validators.required] }),

            // Подрядчик
            contractor_name: new FormControl(this.defaultData.contractor_name, { nonNullable: true, validators: [Validators.required] }),
            contractor_position: new FormControl(this.defaultData.contractor_position, { nonNullable: true, validators: [Validators.required] }),
            contractor_signatory_name: new FormControl(this.defaultData.contractor_signatory_name, { nonNullable: true, validators: [Validators.required] }),
            contractor_genitive: new FormControl(this.defaultData.contractor_genitive, { nonNullable: true, validators: [Validators.required] }),
            contractor_basis: new FormControl(this.defaultData.contractor_basis, { nonNullable: true, validators: [Validators.required] }),
            contractor_reg_info: new FormControl(this.defaultData.contractor_reg_info, { nonNullable: true }),
            contractor_address: new FormControl(this.defaultData.contractor_address, { nonNullable: true, validators: [Validators.required] }),
            contractor_id_type: new FormControl(this.defaultData.contractor_id_type, { nonNullable: true, validators: [Validators.required] }),
            contractor_iin_bin: new FormControl(this.defaultData.contractor_iin_bin, { nonNullable: true, validators: [Validators.required] }),
            contractor_kbe: new FormControl(this.defaultData.contractor_kbe, { nonNullable: true }),
            contractor_iik: new FormControl(this.defaultData.contractor_iik, { nonNullable: true, validators: [Validators.required] }),
            contractor_bank: new FormControl(this.defaultData.contractor_bank, { nonNullable: true, validators: [Validators.required] }),
            contractor_bik: new FormControl(this.defaultData.contractor_bik, { nonNullable: true, validators: [Validators.required] }),

            preamble_closing: new FormControl(this.defaultData.preamble_closing, { nonNullable: true, validators: [Validators.required] }),
            body_text: new FormControl(this.defaultData.body_text, { nonNullable: true, validators: [Validators.required] }),
        });

        this.generatePages(this.form.getRawValue());

        this.form.valueChanges.pipe(debounceTime(300)).subscribe(() => {
            this.generatePages(this.form.getRawValue());
        });
    }

    private generatePages(data: AgreementTemplateData): void {
        const CHARS_PER_LINE = 75;
        const LINES_PER_PAGE = 36;
        const HEADER_LINES = 14;
        const SIGNATURE_LINES = 22;

        const generatedPages: PreviewPage[] = [];
        let currentPage: PreviewPage = {
            pageNumber: 1,
            showHeader: true,
            paragraphs: [],
            showSignatures: false
        };

        let currentLines = HEADER_LINES;
        const rawHtml = data.body_text || '';

        // Захватываем блоки: целые списки, заголовки, параграфы и цитаты
        const matches = rawHtml.match(/<(ol|ul|p|h[1-6]|blockquote)[^>]*>[\s\S]*?<\/\1>/gi) || [rawHtml];

        for (const htmlBlock of matches) {
            const textOnly = htmlBlock.replace(/<[^>]*>/g, '').trim();

            const isHeading = /^<h/i.test(htmlBlock);
            const isList = /^<(ol|ul)/i.test(htmlBlock);

            let pLines: number;
            if (textOnly.length === 0) {
                pLines = 1.2;
            } else if (isHeading) {
                pLines = Math.ceil(textOnly.length / CHARS_PER_LINE) + 1.8;
            } else if (isList) {
                pLines = Math.ceil(textOnly.length / (CHARS_PER_LINE - 10)) + 1.0;
            } else {
                pLines = Math.ceil(textOnly.length / CHARS_PER_LINE) + 1.0;
            }

            if (currentLines + pLines > LINES_PER_PAGE) {
                generatedPages.push(currentPage);
                currentPage = {
                    pageNumber: generatedPages.length + 1,
                    showHeader: false,
                    paragraphs: [],
                    showSignatures: false
                };
                currentLines = 0;
            }

            currentPage.paragraphs.push(htmlBlock);
            currentLines += pLines;
        }

        if (currentLines + SIGNATURE_LINES > LINES_PER_PAGE) {
            generatedPages.push(currentPage);
            generatedPages.push({
                pageNumber: generatedPages.length + 1,
                showHeader: false,
                paragraphs: [],
                showSignatures: true
            });
        } else {
            currentPage.showSignatures = true;
            generatedPages.push(currentPage);
        }

        this.pages.set(generatedPages);
    }

    async onGenerate(): Promise<void> {
        if (this.form.invalid) return;
        this.isGenerating.set(true);
        try {
            const formData = this.form.getRawValue();
            const ext = this.format() === 'pdf' ? 'pdf' : 'docx';

            const result = await this.docGenService.generateDocument<AgreementTemplateData>({
                templateId: 'agreement',
                extension: ext,
                filterName: ext.toUpperCase(),
                defaultFileName: `Договор_${formData.agreement_number}.${ext}`,
                data: formData
            });
            if (result) console.log('Договор успешно сохранен:', result);
        } catch (e) {
            console.error('Ошибка сборки договора:', e);
        } finally {
            this.isGenerating.set(false);
        }
    }
}