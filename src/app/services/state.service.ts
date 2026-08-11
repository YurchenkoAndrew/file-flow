import {Service, signal} from '@angular/core';

@Service()
export class StateService {
    // Храним текущую папку, которую сканировали
    readonly lastScannedPath = signal<string>('');
}
