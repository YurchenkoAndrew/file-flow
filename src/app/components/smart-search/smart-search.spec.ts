import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SmartSearch } from './smart-search';

describe('SmartSearch', () => {
  let component: SmartSearch;
  let fixture: ComponentFixture<SmartSearch>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SmartSearch],
    }).compileComponents();

    fixture = TestBed.createComponent(SmartSearch);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
