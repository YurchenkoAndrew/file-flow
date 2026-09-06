import {Component, inject, input, OnInit, signal} from '@angular/core';
import {MatButton} from "@angular/material/button";
import {MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {FormBuilder, FormControl, FormGroup, ReactiveFormsModule, Validators} from "@angular/forms";
import {DateHelper} from "../../../../helpers/date.helper";
import {DocumentGeneratorService} from "../../../../services/document-generator.service";

export interface ApplicationTemplateData {
    recipient_role: string;
    recipient_name: string;
    sender_role: string;
    sender_name: string;
    title: string;
    body_text: string;
    date: string;
    signer_name: string;
}

// Описываем строгую форму для TypeScript
export interface ApplicationFormModel {
    recipient_role: FormControl<string>;
    recipient_name: FormControl<string>;
    sender_role: FormControl<string>;
    sender_name: FormControl<string>;
    title: FormControl<string>;
    body_text: FormControl<string>;
    date: FormControl<string>;
    signer_name: FormControl<string>;
}

@Component({
    selector: 'app-application-template',
    imports: [
        MatButton,
        MatFormField,
        MatInput,
        MatLabel,
        ReactiveFormsModule
    ],
    templateUrl: './application-template.html',
    styleUrl: './application-template.css',
})
export class ApplicationTemplate implements OnInit {
    private fb = inject(FormBuilder);
    private docGenService = inject(DocumentGeneratorService)
    format = input.required<string>();

    // Типизируем форму через интерфейс
    form!: FormGroup<ApplicationFormModel>;
    isGenerating = signal(false);

    defaultData: ApplicationTemplateData = {
        recipient_role: 'Директору ТОО "File Flow"',
        recipient_name: 'Иванову И. И.',
        sender_role: 'от инженера-разработчика',
        sender_name: 'Юрченко А.',
        title: 'ЗАЯВЛЕНИЕ',
        body_text: 'Прошу предоставить мне ежегодный оплачиваемый отпуск с 15 сентября 2026 года сроком на 14 календарных дней.',
        date: '02.09.2026',
        signer_name: 'Юрченко А.'
    }

    ngOnInit(): void {
        // Создаем форму с непустыми значениями и nonNullable, чтобы getValue всегда возвращал строку
        this.form = this.fb.group({
            recipient_role: new FormControl(this.defaultData.recipient_role, {
                nonNullable: true,
                validators: [Validators.required]
            }),
            recipient_name: new FormControl(this.defaultData.recipient_name, {
                nonNullable: true,
                validators: [Validators.required]
            }),
            sender_role: new FormControl(this.defaultData.sender_role, {
                nonNullable: true,
                validators: [Validators.required]
            }),
            sender_name: new FormControl(this.defaultData.sender_name, {
                nonNullable: true,
                validators: [Validators.required]
            }),
            title: new FormControl(this.defaultData.title, {nonNullable: true, validators: [Validators.required]}),
            body_text: new FormControl(this.defaultData.body_text, {
                nonNullable: true,
                validators: [Validators.required]
            }),
            date: new FormControl(DateHelper.today(), {nonNullable: true, validators: [Validators.required]}),
            signer_name: new FormControl(this.defaultData.signer_name, {
                nonNullable: true,
                validators: [Validators.required]
            }),
        });
    }

    async onGenerate() {
        if (this.form.invalid) return;
        this.isGenerating.set(true);
        try {
            const formData = this.form.value as ApplicationTemplateData;

            // Всего два формата: если не PDF, значит DOCX
            const formatSelected = this.format();
            const isPdf = formatSelected === 'pdf'; // или твой ID для PDF
            const ext = isPdf ? 'pdf' : 'docx';
            const filterTitle = isPdf ? 'PDF' : 'Word';

            const result = await this.docGenService.generateDocument<ApplicationTemplateData>({
                templateId: 'application',
                extension: ext,
                filterName: filterTitle,
                defaultFileName: `Заявление_${formData.signer_name.replace(/\s+/g, '_')}.${ext}`,
                data: formData
            });

            if (result) {
                console.log('Документ успешно сохранен по пути:', result);
            }
        } catch (e) {
            console.error('Ошибка сборки:', e);
        } finally {
            this.isGenerating.set(false);
        }
    }

}
