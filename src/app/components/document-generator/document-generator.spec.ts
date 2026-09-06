import { ComponentFixture, TestBed } from '@angular/core/testing';

import { DocumentGenerator } from './document-generator';

describe('DocumentGenerator', () => {
  let component: DocumentGenerator;
  let fixture: ComponentFixture<DocumentGenerator>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DocumentGenerator],
    }).compileComponents();

    fixture = TestBed.createComponent(DocumentGenerator);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
