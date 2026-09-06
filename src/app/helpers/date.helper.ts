import {Service} from '@angular/core';

@Service()
export class DateHelper {
    public static today(): string {
        // Получаем текущую дату с клиентской машины
        const today = new Date();
        return new Intl.DateTimeFormat('ru-RU', {
            day: '2-digit',
            month: '2-digit',
            year: 'numeric'
        }).format(today);
    }
}
