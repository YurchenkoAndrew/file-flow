import {Routes} from '@angular/router';

export const routes: Routes = [
    {path: '', redirectTo: '/scanner', pathMatch: 'full'},
    {
        path: 'scanner',
        loadComponent: () => import('./components/scanner/scanner').then(m => m.Scanner)
    },
    {
        path: 'sorter',
        loadComponent: () => import('./components/sorter/sorter').then(m => m.Sorter)
    },
    {
        path: 'smart-search',
        loadComponent: () => import('./components/smart-search/smart-search').then(m => m.SmartSearch)
    },
    {
        path: 'document-generator',
        loadComponent: () => import('./components/document-generator/document-generator').then(m => m.DocumentGenerator)
    },
    {
        path: 'about',
        loadComponent: () => import('./components/about/about').then(m => m.About)
    },
];