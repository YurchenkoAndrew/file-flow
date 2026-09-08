import {Component, inject, input, OnInit, signal} from '@angular/core';
import {FormBuilder, FormControl, FormGroup, ReactiveFormsModule, Validators} from "@angular/forms";
import {DocumentGeneratorService} from "../../../../services/document-generator.service";
import {DateHelper} from "../../../../helpers/date.helper";
import {MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {MatButton} from "@angular/material/button";

export interface MemoTemplateData {
    recipient_role: string;
    recipient_name: string;
    sender_department: string;
    sender_role: string;
    sender_name: string;
    subject: string;
    body_text: string;
    date: string;
    signer_name: string;
}

export interface MemoFormModel {
    recipient_role: FormControl<string>;
    recipient_name: FormControl<string>;
    sender_department: FormControl<string>;
    sender_role: FormControl<string>;
    sender_name: FormControl<string>;
    subject: FormControl<string>;
    body_text: FormControl<string>;
    date: FormControl<string>;
    signer_name: FormControl<string>;
}

@Component({
    selector: 'app-memo-template',
  imports: [
    ReactiveFormsModule,
    MatFormField,
    MatLabel,
    MatInput,
    MatButton
  ],
    templateUrl: './memo-template.html',
    styleUrl: './memo-template.css',
})
export class MemoTemplate implements OnInit {
    private fb = inject(FormBuilder);
    private docGenService = inject(DocumentGeneratorService);

    format = input.required<string>();

    form!: FormGroup<MemoFormModel>;
    isGenerating = signal(false);

    defaultData: MemoTemplateData = {
        recipient_role: 'Генеральному директору ТОО "File Flow"',
        recipient_name: 'Иванову И. И.',
        sender_department: 'Отдел разработки',
        sender_role: 'от системного архитектора',
        sender_name: 'Юрченко А.',
        subject: 'О выделении серверных мощностей',
        body_text: 'В связи с расширением функционала проекта и увеличением нагрузки на тестовый контур, прошу выделить дополнительные серверные мощности (4 CPU, 16 GB RAM, 100 GB SSD).',
        date: DateHelper.today(),
        signer_name: 'Юрченко А.'
    }

    ngOnInit(): void {
        this.form = this.fb.group({
            recipient_role: new FormControl(this.defaultData.recipient_role, {
                nonNullable: true, validators: [Validators.required]
            }),
            recipient_name: new FormControl(this.defaultData.recipient_name, {
                nonNullable: true, validators: [Validators.required]
            }),
            sender_department: new FormControl(this.defaultData.sender_department, {
                nonNullable: true
            }),
            sender_role: new FormControl(this.defaultData.sender_role, {
                nonNullable: true, validators: [Validators.required]
            }),
            sender_name: new FormControl(this.defaultData.sender_name, {
                nonNullable: true, validators: [Validators.required]
            }),
            subject: new FormControl(this.defaultData.subject, {
                nonNullable: true
            }),
            body_text: new FormControl(this.defaultData.body_text, {
                nonNullable: true, validators: [Validators.required]
            }),
            date: new FormControl(this.defaultData.date, {
                nonNullable: true, validators: [Validators.required]
            }),
            signer_name: new FormControl(this.defaultData.signer_name, {
                nonNullable: true, validators: [Validators.required]
            }),
        });
    }

    async onGenerate() {
        if (this.form.invalid) return;
        this.isGenerating.set(true);
        try {
            const formData = this.form.value as MemoTemplateData;

            const formatSelected = this.format();
            const isPdf = formatSelected === 'pdf';
            const ext = isPdf ? 'pdf' : 'docx';
            const filterTitle = isPdf ? 'PDF' : 'Word';

            const result = await this.docGenService.generateDocument<MemoTemplateData>({
                templateId: 'memo',
                extension: ext,
                filterName: filterTitle,
                defaultFileName: `Служебная_записка_${formData.signer_name.replace(/\s+/g, '_')}.${ext}`,
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
