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

    // Генерация текста заявления с будущей датой (например, через 7 дней)
    public static getApplicationBody(daysToAdd: number = 7): string {
        const targetDate = new Date();
        targetDate.setDate(targetDate.getDate() + daysToAdd);

        return new Intl.DateTimeFormat('ru-RU', {
            day: 'numeric',
            month: 'long',
            year: 'numeric'
        }).format(targetDate).replace(' г.', ' года');
    }
}
