import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AgreementTemplate } from './agreement-template';

describe('AgreementTemplate', () => {
  let component: AgreementTemplate;
  let fixture: ComponentFixture<AgreementTemplate>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgreementTemplate],
    }).compileComponents();

    fixture = TestBed.createComponent(AgreementTemplate);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
