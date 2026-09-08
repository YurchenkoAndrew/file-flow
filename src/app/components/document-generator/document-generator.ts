import {Component, computed, OnInit} from '@angular/core';
import {FormControl, FormGroup, FormsModule, ReactiveFormsModule} from "@angular/forms";
import {DOCUMENT_FORMATS, DOCUMENT_TYPES} from "./templates/document-types.const";
import {MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {MatAutocomplete, MatAutocompleteTrigger, MatOption} from "@angular/material/autocomplete";
import {MatRadioButton, MatRadioGroup} from "@angular/material/radio";
import {ApplicationTemplate} from "./templates/application-template/application-template";
import {toSignal} from "@angular/core/rxjs-interop";
import {MemoTemplate} from "./templates/memo-template/memo-template";

interface DocumentType {
    id: string;
    name: string;
}

@Component({
    selector: 'app-document-generator',
    imports: [
        FormsModule,
        ReactiveFormsModule,
        MatFormField,
        MatLabel,
        MatInput,
        MatAutocomplete,
        MatOption,
        MatAutocompleteTrigger,
        MatRadioGroup,
        MatRadioButton,
        ApplicationTemplate,
        MemoTemplate
    ],
    templateUrl: './document-generator.html',
    styleUrl: './document-generator.css',
})
export class DocumentGenerator implements OnInit {
    documentTypes = DOCUMENT_TYPES;
    formats = DOCUMENT_FORMATS;

    form = new FormGroup({
        selectedDoc: new FormControl<string | DocumentType>(''),
        selectedFormat: new FormControl<string>('pdf', {nonNullable: true}),
    });

    ngOnInit() {
    }

    // 1. Читаем каждое нажатие клавиши из формы
    searchQuery = toSignal(this.form.controls.selectedDoc.valueChanges, {initialValue: ''});

    // 2. Сигнал, который налету фильтрует массив
    filteredDocuments = computed(() => {
        const value = this.searchQuery();

        // Достаем строку: либо то, что пользователь печатает, либо имя уже выбранного объекта
        const searchStr = typeof value === 'string' ? value : (value?.name || '');

        // Если поле пустое — отдаем весь список
        if (!searchStr.trim()) {
            return this.documentTypes;
        }

        // Фильтруем без учета регистра
        const lowerCaseQuery = searchStr.toLowerCase();
        return this.documentTypes.filter(doc =>
            doc.name.toLowerCase().includes(lowerCaseQuery)
        );
    });

    // Геттер для динамического подтягивания шаблона снизу
    get selectedDocId(): string | null {
        const val = this.form.get('selectedDoc')?.value;
        return typeof val === 'object' && val !== null ? val.id : null;
    }

    displayFn(doc: DocumentType): string {
        return doc && doc.name ? doc.name : '';
    }
}
